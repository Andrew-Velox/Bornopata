//! The state of the panel thread and its event loop.

use std::sync::mpsc;
use std::time::{Duration, Instant};

use smithay_client_toolkit::compositor::{CompositorState, Region};
use smithay_client_toolkit::output::OutputState;
use smithay_client_toolkit::reexports::calloop::EventLoop;
use smithay_client_toolkit::reexports::calloop::channel::Event;
use smithay_client_toolkit::reexports::calloop_wayland_source::WaylandSource;
use smithay_client_toolkit::reexports::client::globals::registry_queue_init;
use smithay_client_toolkit::reexports::client::protocol::wl_shm;
use smithay_client_toolkit::reexports::client::{Connection, QueueHandle};
use smithay_client_toolkit::registry::RegistryState;
use smithay_client_toolkit::shell::WaylandSurface;
use smithay_client_toolkit::shell::wlr_layer::{
    Anchor, KeyboardInteractivity, Layer, LayerShell, LayerSurface,
};
use smithay_client_toolkit::shm::Shm;
use smithay_client_toolkit::shm::slot::SlotPool;

use borno_render::{FontLibrary, Layout, Rendered, Renderer, Theme};

use crate::buffer::rgba_to_argb8888;
use crate::command::Command;
use crate::compositor::{self, Compositor, CursorRect, Located};
use crate::error::PanelError;
use crate::theme::load_theme;
use borno_window::PanelConfig;
use borno_window::Rect;
use borno_window::placement::place;

/// What the next draw needs once the surface is configured.
pub(super) struct Pending {
    pub rendered: Rendered,
    pub scale: i32,
}

pub(super) struct State {
    pub registry_state: RegistryState,
    pub output_state: OutputState,
    pub compositor_state: CompositorState,
    pub layer_shell: LayerShell,
    pub shm: Shm,
    pub pool: SlotPool,
    pub qh: QueueHandle<State>,

    renderer: Renderer,
    theme: Theme,
    layout: Layout,
    compositor: Option<Box<dyn Compositor>>,

    /// The visible window, if any.
    pub layer: Option<LayerSurface>,
    pub configured: bool,
    pub pending: Option<Pending>,

    /// When the next animation frame is due.
    tick_at: Option<Instant>,
    pub quit: bool,
}

/// Thread body: set everything up, report readiness, then run the event loop until asked to stop.
pub(super) fn run(
    config: PanelConfig,
    commands: smithay_client_toolkit::reexports::calloop::channel::Channel<Command>,
    ready: mpsc::Sender<Result<(), PanelError>>,
) {
    match setup(config) {
        Ok((mut state, mut event_loop)) => {
            let handle = event_loop.handle();
            let inserted =
                handle.insert_source(commands, |event, _, state: &mut State| match event {
                    Event::Msg(command) => state.command(command),
                    Event::Closed => state.quit = true,
                });
            if let Err(error) = inserted {
                let _ = ready.send(Err(PanelError::Protocol(error.to_string())));
                return;
            }
            let _ = ready.send(Ok(()));
            while !state.quit {
                let timeout = state.next_timeout();
                if let Err(error) = event_loop.dispatch(timeout, &mut state) {
                    tracing::warn!(%error, "the panel event loop stopped");
                    break;
                }
                state.tick_if_due();
            }
        }
        Err(error) => {
            let _ = ready.send(Err(error));
        }
    }
}

fn setup(config: PanelConfig) -> Result<(State, EventLoop<'static, State>), PanelError> {
    let connection =
        Connection::connect_to_env().map_err(|error| PanelError::Connect(error.to_string()))?;
    let (globals, event_queue) = registry_queue_init(&connection)
        .map_err(|error| PanelError::Protocol(error.to_string()))?;
    let qh = event_queue.handle();
    let compositor_state =
        CompositorState::bind(&globals, &qh).map_err(|_| PanelError::Missing("wl_compositor"))?;
    let layer_shell =
        LayerShell::bind(&globals, &qh).map_err(|_| PanelError::Missing("wlr-layer-shell"))?;
    let shm = Shm::bind(&globals, &qh).map_err(|_| PanelError::Missing("wl_shm"))?;
    let pool =
        SlotPool::new(64 * 1024, &shm).map_err(|error| PanelError::Protocol(error.to_string()))?;

    let library =
        FontLibrary::system("en").map_err(|error| PanelError::Render(error.to_string()))?;
    let mut renderer = Renderer::new(library);
    let theme = load_theme(&config)?;
    renderer.load_theme_fonts(&theme, |_| Vec::new());

    let event_loop: EventLoop<State> =
        EventLoop::try_new().map_err(|error| PanelError::Protocol(error.to_string()))?;
    WaylandSource::new(connection, event_queue)
        .insert(event_loop.handle())
        .map_err(|error| PanelError::Protocol(error.to_string()))?;
    let state = State {
        registry_state: RegistryState::new(&globals),
        output_state: OutputState::new(&globals, &qh),
        compositor_state,
        layer_shell,
        shm,
        pool,
        qh,
        renderer,
        theme,
        layout: config.layout,
        compositor: compositor::detect(),
        layer: None,
        configured: false,
        pending: None,
        tick_at: None,
        quit: false,
    };
    Ok((state, event_loop))
}

impl State {
    pub(super) fn command(&mut self, command: Command) {
        match command {
            Command::Show { frame, cursor } => self.show(&frame, cursor),
            Command::Hide => self.hide(),
            Command::Shutdown => self.quit = true,
        }
    }

    fn show(&mut self, frame: &borno_render::Frame, cursor: CursorRect) {
        if frame.is_empty() {
            self.hide();
            return;
        }
        let located = self
            .compositor
            .as_mut()
            .and_then(|compositor| compositor.locate(cursor))
            .unwrap_or_else(|| self.fallback_location(cursor));
        let scale = located.scale.ceil().max(1.0) as i32;
        let rendered = match self
            .renderer
            .render(frame, self.layout, &self.theme, scale as f32)
        {
            Ok(rendered) => rendered,
            Err(error) => {
                tracing::warn!(%error, "could not draw the candidate window");
                return;
            }
        };
        let factor = scale as f32;
        let content = (
            rendered.content_width as f32 / factor,
            rendered.content_height as f32 / factor,
        );
        let (x, y) = place(located.cursor, content, located.monitor);
        // The bitmap reaches beyond the card (shadow, decorations): the surface origin is the content origin
        // minus that margin, in the monitor's own coordinates.
        let left = x - rendered.content_x as f32 / factor - located.monitor.x;
        let top = y - rendered.content_y as f32 / factor - located.monitor.y;
        let size = (
            (rendered.pixmap.width() as f32 / factor).ceil() as u32,
            (rendered.pixmap.height() as f32 / factor).ceil() as u32,
        );
        self.tick_at = rendered.next_frame.map(|after| Instant::now() + after);
        self.pending = Some(Pending { rendered, scale });
        let layer = self.ensure_layer(&located.monitor_name);
        layer.set_size(size.0, size.1);
        layer.set_margin(top.round() as i32, 0, 0, left.round() as i32);
        if self.configured {
            self.draw();
        } else {
            self.layer.as_ref().expect("just created").commit();
        }
    }

    /// Without compositor geometry the cursor is taken as global on a large monitor; the popup still shows, just
    /// not necessarily where the cursor is.
    fn fallback_location(&self, cursor: CursorRect) -> Located {
        Located {
            cursor: cursor.rect,
            monitor: Rect::new(0.0, 0.0, 1920.0, 1080.0),
            monitor_name: String::new(),
            scale: 1.0,
        }
    }

    fn ensure_layer(&mut self, monitor: &str) -> &LayerSurface {
        if self.layer.is_none() {
            let output = self.output_state.outputs().find(|output| {
                self.output_state
                    .info(output)
                    .and_then(|info| info.name)
                    .is_some_and(|name| name == monitor)
            });
            let surface = self.compositor_state.create_surface(&self.qh);
            let layer = self.layer_shell.create_layer_surface(
                &self.qh,
                surface,
                Layer::Overlay,
                Some("bornopata"),
                output.as_ref(),
            );
            layer.set_anchor(Anchor::TOP | Anchor::LEFT);
            layer.set_keyboard_interactivity(KeyboardInteractivity::None);
            layer.set_exclusive_zone(-1);
            if let Ok(region) = Region::new(&self.compositor_state) {
                layer.set_input_region(Some(region.wl_region()));
            }
            self.configured = false;
            self.layer = Some(layer);
        }
        self.layer.as_ref().expect("created above")
    }

    /// Copies the pending bitmap into a shared-memory buffer and shows it.
    pub(super) fn draw(&mut self) {
        let (Some(layer), Some(pending)) = (self.layer.as_ref(), self.pending.as_ref()) else {
            return;
        };
        let pixmap = &pending.rendered.pixmap;
        let (width, height) = (pixmap.width() as i32, pixmap.height() as i32);
        let Ok((buffer, canvas)) =
            self.pool
                .create_buffer(width, height, width * 4, wl_shm::Format::Argb8888)
        else {
            tracing::warn!("could not allocate a shared-memory buffer");
            return;
        };
        let converted = rgba_to_argb8888(pixmap.data());
        canvas[..converted.len()].copy_from_slice(&converted);
        let surface = layer.wl_surface();
        surface.set_buffer_scale(pending.scale);
        surface.damage_buffer(0, 0, width, height);
        if buffer.attach_to(surface).is_err() {
            return;
        }
        layer.commit();
    }

    fn hide(&mut self) {
        self.layer = None;
        self.configured = false;
        self.pending = None;
        self.tick_at = None;
        self.renderer.forget();
    }

    fn next_timeout(&self) -> Option<Duration> {
        self.tick_at
            .map(|due| due.saturating_duration_since(Instant::now()))
    }

    fn tick_if_due(&mut self) {
        let Some(due) = self.tick_at else { return };
        if Instant::now() < due || self.layer.is_none() || !self.configured {
            return;
        }
        match self.renderer.tick() {
            Ok(Some(rendered)) => {
                self.tick_at = rendered.next_frame.map(|after| Instant::now() + after);
                if let Some(pending) = self.pending.as_mut() {
                    pending.rendered = rendered;
                }
                self.draw();
            }
            Ok(None) => self.tick_at = None,
            Err(error) => {
                tracing::warn!(%error, "animation frame failed");
                self.tick_at = None;
            }
        }
    }
}
