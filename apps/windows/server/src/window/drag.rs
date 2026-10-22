//! Dragging the window with the mouse. The window procedure moves the window while the left button is held and leaves
//! a note; the window thread reads it after the message and keeps the pin in step.

use std::cell::Cell;

use windows::Win32::Foundation::{HWND, POINT, RECT};
use windows::Win32::UI::Input::KeyboardAndMouse::{ReleaseCapture, SetCapture};
use windows::Win32::UI::WindowsAndMessaging::{
    GetCursorPos, GetWindowRect, SWP_NOACTIVATE, SWP_NOSIZE, SWP_NOZORDER, SetWindowPos,
};

/// What a mouse message did to the window.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum Moved {
    /// The window is now at this corner, the button still held.
    To(i32, i32),

    /// The button was let go with the window at this corner.
    Dropped(i32, i32),

    /// The right button: the window should follow the text cursor again.
    Freed,
}

#[derive(Clone, Copy)]
struct Grab {
    /// Where the mouse was on the screen when the button went down, and where the window's corner was.
    cursor: POINT,
    corner: POINT,
    moved: bool,
}

thread_local! {
    static GRAB: Cell<Option<Grab>> = const { Cell::new(None) };
    static NOTE: Cell<Option<Moved>> = const { Cell::new(None) };
}

/// What the last message did, once.
pub(super) fn take() -> Option<Moved> {
    NOTE.with(Cell::take)
}

pub(super) fn press(hwnd: HWND) {
    let mut cursor = POINT::default();
    let mut rect = RECT::default();
    if unsafe { GetCursorPos(&mut cursor) }.is_err()
        || unsafe { GetWindowRect(hwnd, &mut rect) }.is_err()
    {
        return;
    }
    unsafe { SetCapture(hwnd) };
    GRAB.with(|grab| {
        grab.set(Some(Grab {
            cursor,
            corner: POINT {
                x: rect.left,
                y: rect.top,
            },
            moved: false,
        }));
    });
}

pub(super) fn motion(hwnd: HWND) {
    let Some(mut grab) = GRAB.with(Cell::get) else {
        return;
    };
    let mut cursor = POINT::default();
    if unsafe { GetCursorPos(&mut cursor) }.is_err() {
        return;
    }
    let x = grab.corner.x + cursor.x - grab.cursor.x;
    let y = grab.corner.y + cursor.y - grab.cursor.y;
    if unsafe {
        SetWindowPos(
            hwnd,
            None,
            x,
            y,
            0,
            0,
            SWP_NOSIZE | SWP_NOZORDER | SWP_NOACTIVATE,
        )
    }
    .is_ok()
    {
        grab.moved = true;
        GRAB.with(|slot| slot.set(Some(grab)));
        NOTE.with(|note| note.set(Some(Moved::To(x, y))));
    }
}

pub(super) fn release(hwnd: HWND) {
    let Some(grab) = GRAB.with(Cell::take) else {
        return;
    };
    let _ = unsafe { ReleaseCapture() };
    if !grab.moved {
        return;
    }
    let mut rect = RECT::default();
    if unsafe { GetWindowRect(hwnd, &mut rect) }.is_ok() {
        NOTE.with(|note| note.set(Some(Moved::Dropped(rect.left, rect.top))));
    }
}

/// Something else took the mouse away (another window, Alt+Tab): the drag is over where it stands.
pub(super) fn lost(hwnd: HWND) {
    release(hwnd);
}

pub(super) fn free() {
    NOTE.with(|note| note.set(Some(Moved::Freed)));
}
