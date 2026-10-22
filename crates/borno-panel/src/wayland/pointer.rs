//! The mouse on the window: drag it to pin it where it is dropped, right-click to let it follow the text cursor.

use smithay_client_toolkit::reexports::client::protocol::wl_pointer::WlPointer;
use smithay_client_toolkit::reexports::client::protocol::wl_seat;
use smithay_client_toolkit::reexports::client::{Connection, QueueHandle};
use smithay_client_toolkit::seat::pointer::{
    BTN_LEFT, BTN_RIGHT, CursorIcon, PointerEvent, PointerEventKind, PointerHandler, ThemeSpec,
};
use smithay_client_toolkit::seat::{Capability, SeatHandler, SeatState};
use smithay_client_toolkit::shell::WaylandSurface;

use super::state::State;

/// A drag in progress. The window is moved by how far the pointer is from where it took hold, in the window's own
/// coordinates: when the compositor keeps up, the pointer stays at that spot on the window.
pub(super) struct Drag {
    grab: (f64, f64),

    /// The latest pointer position, until the end of the frame of events applies it.
    latest: Option<(f64, f64)>,

    moved: bool,
}

impl SeatHandler for State {
    fn seat_state(&mut self) -> &mut SeatState {
        &mut self.seat_state
    }

    fn new_seat(&mut self, _: &Connection, _: &QueueHandle<Self>, _: wl_seat::WlSeat) {}

    fn new_capability(
        &mut self,
        _: &Connection,
        qh: &QueueHandle<Self>,
        seat: wl_seat::WlSeat,
        capability: Capability,
    ) {
        if capability != Capability::Pointer || self.pointer.is_some() {
            return;
        }
        let surface = self.compositor_state.create_surface(qh);
        match self.seat_state.get_pointer_with_theme_and_data(
            qh,
            &seat,
            self.shm.wl_shm(),
            surface,
            ThemeSpec::System,
            (),
        ) {
            Ok(pointer) => self.pointer = Some(pointer),
            Err(error) => tracing::warn!(%error, "the window cannot see the mouse"),
        }
    }

    fn remove_capability(
        &mut self,
        _: &Connection,
        _: &QueueHandle<Self>,
        _: wl_seat::WlSeat,
        capability: Capability,
    ) {
        if capability == Capability::Pointer {
            self.pointer = None;
            self.drag = None;
        }
    }

    fn remove_seat(&mut self, _: &Connection, _: &QueueHandle<Self>, _: wl_seat::WlSeat) {}
}

impl PointerHandler for State {
    fn pointer_frame(
        &mut self,
        conn: &Connection,
        _: &QueueHandle<Self>,
        _: &WlPointer,
        events: &[PointerEvent],
    ) {
        let ours = self.layer.as_ref().map(|layer| layer.wl_surface().clone());
        for event in events {
            if self.drag.is_none() && ours.as_ref() != Some(&event.surface) {
                continue;
            }
            match event.kind {
                PointerEventKind::Enter { .. } => {
                    if let Some(pointer) = &self.pointer {
                        let _ = pointer.set_cursor(conn, CursorIcon::Grab);
                    }
                }
                PointerEventKind::Press { button, .. } if button == BTN_LEFT => {
                    self.drag = Some(Drag {
                        grab: event.position,
                        latest: None,
                        moved: false,
                    });
                }
                PointerEventKind::Press { button, .. } if button == BTN_RIGHT => {
                    self.release_pin();
                }
                PointerEventKind::Motion { .. } => {
                    if let Some(drag) = self.drag.as_mut() {
                        drag.latest = Some(event.position);
                    }
                }
                PointerEventKind::Release { button, .. } if button == BTN_LEFT => {
                    self.apply_drag();
                    if let Some(drag) = self.drag.take()
                        && drag.moved
                        && let Err(error) = self.pin.save()
                    {
                        tracing::warn!(%error, "could not save where the window was dropped");
                    }
                }
                _ => {}
            }
        }
        self.apply_drag();
    }
}

impl State {
    /// Moves the window by what the pointer travelled since the last move.
    fn apply_drag(&mut self) {
        let Some(drag) = self.drag.as_mut() else {
            return;
        };
        let Some(latest) = drag.latest.take() else {
            return;
        };
        let (dx, dy) = (
            (latest.0 - drag.grab.0) as f32,
            (latest.1 - drag.grab.1) as f32,
        );
        if dx == 0.0 && dy == 0.0 {
            return;
        }
        drag.moved = true;
        self.move_card_by(dx, dy);
    }

    /// Right click: the window follows the text cursor again from the next word on.
    fn release_pin(&mut self) {
        if let Err(error) = self.pin.clear() {
            tracing::warn!(%error, "could not forget where the window was pinned");
        }
    }
}
