//! The render thread's state: what the popup shows, the pools of each shell channel, and the animation clock.

use std::collections::{HashMap, HashSet};
use std::os::fd::AsFd;
use std::os::unix::net::UnixStream;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use borno_platform::protocol::{PopupFrame, PopupServerMessage};
use borno_render::{Frame, Layout, Rendered, Renderer, Theme};

use super::pixels::copy_cropped;
use super::pool::Pool;
use super::wire::send_message;

/// Scales the compositor may ask for, in 120ths: 1.0 to 6.0.
const SCALES: std::ops::RangeInclusive<u32> = 120..=720;

const WRITE_TIMEOUT: Duration = Duration::from_secs(1);

/// What one shell channel has: its socket, the scale it asked for and the memory the Server draws into.
struct Channel {
    stream: UnixStream,
    scale_120: u32,
    pools: Vec<Pool>,

    /// Buffer number to the pool and slot it occupies, until the shell releases it.
    in_flight: HashMap<u64, (u64, usize)>,

    /// A bitmap that found no free slot; it is shown as soon as one comes back.
    deferred: Option<Rendered>,
}

/// The frame on screen, kept to draw again when the scale changes.
struct Active {
    token: u64,
    context: String,
    frame: Frame,
}

pub(super) struct Worker {
    renderer: Renderer,
    theme: Theme,
    layout: Layout,
    attached: Arc<Mutex<HashSet<u64>>>,
    channels: HashMap<u64, Channel>,
    active: Option<Active>,
    next_buffer: u64,
    next_pool: u64,

    /// When the next animation frame is due.
    tick_at: Option<Instant>,
}

impl Worker {
    pub(super) fn new(
        renderer: Renderer,
        theme: Theme,
        layout: Layout,
        attached: Arc<Mutex<HashSet<u64>>>,
    ) -> Self {
        Self {
            renderer,
            theme,
            layout,
            attached,
            channels: HashMap::new(),
            active: None,
            next_buffer: 1,
            next_pool: 1,
            tick_at: None,
        }
    }

    pub(super) fn attach(&mut self, token: u64, stream: UnixStream) {
        let _ = stream.set_write_timeout(Some(WRITE_TIMEOUT));
        self.attached.lock().unwrap().insert(token);
        let replaced = self.channels.insert(
            token,
            Channel {
                stream,
                scale_120: *SCALES.start(),
                pools: Vec::new(),
                in_flight: HashMap::new(),
                deferred: None,
            },
        );
        // A shell that attaches again (it restarted) starts from nothing.
        if replaced.is_some() && self.active.as_ref().is_some_and(|a| a.token == token) {
            self.clear_active();
        }
    }

    pub(super) fn detach(&mut self, token: u64) {
        self.attached.lock().unwrap().remove(&token);
        if self.channels.remove(&token).is_some()
            && self.active.as_ref().is_some_and(|a| a.token == token)
        {
            self.clear_active();
        }
    }

    pub(super) fn released(&mut self, token: u64, buffer: u64) {
        let Some(channel) = self.channels.get_mut(&token) else {
            return;
        };
        if let Some((pool, slot)) = channel.in_flight.remove(&buffer)
            && let Some(pool) = channel.pools.iter_mut().find(|p| p.id() == pool)
        {
            pool.release(slot);
        }
        let current = channel.pools.last().map(Pool::id);
        channel
            .pools
            .retain(|pool| Some(pool.id()) == current || !pool.is_idle());
        if let Some(rendered) = channel.deferred.take()
            && self.active.as_ref().is_some_and(|a| a.token == token)
        {
            self.present(rendered);
        }
    }

    pub(super) fn scale(&mut self, token: u64, scale_120: u32) {
        let Some(channel) = self.channels.get_mut(&token) else {
            return;
        };
        let scale_120 = scale_120.clamp(*SCALES.start(), *SCALES.end());
        if channel.scale_120 == scale_120 {
            return;
        }
        channel.scale_120 = scale_120;
        if self.active.as_ref().is_some_and(|a| a.token == token) {
            self.draw_active();
        }
    }

    /// Shows `frame` in the popup of channel `token` for input context `context`.
    pub(super) fn show(&mut self, token: u64, context: String, frame: Frame) {
        if frame.is_empty() {
            self.hide(token);
            return;
        }
        if self.active.as_ref().is_some_and(|a| a.token != token) {
            let previous = self.active.as_ref().map(|a| a.token).unwrap_or(token);
            self.hide(previous);
        }
        self.active = Some(Active {
            token,
            context,
            frame,
        });
        self.draw_active();
    }

    pub(super) fn hide(&mut self, token: u64) {
        if !self.active.as_ref().is_some_and(|a| a.token == token) {
            return;
        }
        self.clear_active();
        let failed = self.channels.get_mut(&token).is_some_and(|channel| {
            send_message(&mut channel.stream, &PopupServerMessage::Hide, None).is_err()
        });
        if failed {
            self.detach(token);
        }
    }

    pub(super) fn next_timeout(&self) -> Option<Duration> {
        self.tick_at
            .map(|due| due.saturating_duration_since(Instant::now()))
    }

    /// Draws the next animation frame when it is due. Nothing is scheduled while no animation plays.
    pub(super) fn tick_if_due(&mut self) {
        let Some(due) = self.tick_at else { return };
        if Instant::now() < due || self.active.is_none() {
            return;
        }
        match self.renderer.tick() {
            Ok(Some(rendered)) => self.present(rendered),
            Ok(None) => self.tick_at = None,
            Err(error) => {
                tracing::warn!(%error, "animation frame failed");
                self.tick_at = None;
            }
        }
    }

    fn clear_active(&mut self) {
        self.active = None;
        self.tick_at = None;
        self.renderer.forget();
    }

    /// Renders the active frame at its channel's scale and presents it.
    fn draw_active(&mut self) {
        let Some(active) = self.active.as_ref() else {
            return;
        };
        let Some(channel) = self.channels.get(&active.token) else {
            return;
        };
        let scale = channel.scale_120 as f32 / 120.0;
        match self
            .renderer
            .render(&active.frame, self.layout, &self.theme, scale)
        {
            Ok(rendered) => self.present(rendered),
            Err(error) => tracing::warn!(%error, "could not draw the candidate popup"),
        }
    }

    /// Copies the card of `rendered` into a free slot and tells the shell. A frame that finds no slot waits for one.
    fn present(&mut self, rendered: Rendered) {
        let Some(active) = self.active.as_ref() else {
            return;
        };
        let token = active.token;
        let context = active.context.clone();
        let Some(channel) = self.channels.get_mut(&token) else {
            return;
        };
        self.tick_at = rendered.next_frame.map(|after| Instant::now() + after);
        let (width, height) = (rendered.content_width, rendered.content_height);
        if width == 0 || height == 0 {
            return;
        }
        let stride = width * 4;
        let bytes = stride as usize * height as usize;
        if channel
            .pools
            .last()
            .is_none_or(|pool| pool.slot_size() < bytes)
        {
            let id = self.next_pool;
            self.next_pool += 1;
            let pool = match Pool::new(id, bytes) {
                Ok(pool) => pool,
                Err(error) => {
                    tracing::warn!(%error, "could not allocate popup memory");
                    return;
                }
            };
            let message = PopupServerMessage::Pool {
                pool: id,
                size: pool.size() as u64,
            };
            if send_message(&mut channel.stream, &message, Some(pool.file().as_fd())).is_err() {
                self.detach(token);
                return;
            }
            channel.pools.push(pool);
        }
        let pool = channel.pools.last_mut().expect("a pool was just ensured");
        let Some(slot) = pool.acquire() else {
            channel.deferred = Some(rendered);
            return;
        };
        copy_cropped(
            rendered.pixmap.data(),
            rendered.pixmap.width(),
            (rendered.content_x, rendered.content_y, width, height),
            pool.slot_mut(slot),
        );
        let buffer = self.next_buffer;
        self.next_buffer += 1;
        let scale_120 = channel.scale_120;
        let logical = |pixels: u32| (pixels * 120).div_ceil(scale_120);
        let frame = PopupFrame {
            context,
            pool: pool.id(),
            buffer,
            offset: pool.offset(slot) as u32,
            width,
            height,
            stride,
            scale_120,
            logical_width: logical(width),
            logical_height: logical(height),
        };
        channel.in_flight.insert(buffer, (pool.id(), slot));
        if send_message(
            &mut channel.stream,
            &PopupServerMessage::Present(frame),
            None,
        )
        .is_err()
        {
            self.detach(token);
        }
    }
}
