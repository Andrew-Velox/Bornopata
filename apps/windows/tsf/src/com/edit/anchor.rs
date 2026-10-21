//! Where the text cursor is on the screen, for the Server to put the candidate window next to: the start of the
//! composition (or the selection), the mouse when an application cannot say.
//!
//! WeChat 4.x answers `GetTextExt` of a non-empty range with a one pixel fragment at its end (the first key gets the full
//! line height, for example (…,1062,…,1082) against (…,1063,…,1064)), so the anchor would jump with every key and the window
//! would cover the line. The composition is therefore always collapsed to its start before it is measured (as Weasel
//! does), and a fragment is repaired to the last normal line height.

use std::mem::ManuallyDrop;
use std::sync::atomic::{AtomicI32, Ordering};

use borno_platform::protocol::CursorRect;
use windows::Win32::Foundation::{POINT, RECT};
use windows::Win32::Graphics::Gdi::{MONITOR_DEFAULTTONULL, MonitorFromPoint};
use windows::Win32::UI::TextServices::{
    ITfContext, ITfRange, TF_ANCHOR_START, TF_DEFAULT_SELECTION, TF_SELECTION,
};
use windows::Win32::UI::WindowsAndMessaging::GetCursorPos;
use windows::core::BOOL;

/// The screen rectangle of `range`; the mouse when the application gives nothing usable (some answer with all zeros or
/// an empty rectangle). Every anchor leaves a line in the log: when the window jumps about, that is where to look.
pub(crate) fn anchor_rect(context: &ITfContext, ec: u32, range: &ITfRange) -> CursorRect {
    let (mut rect, source) = collapsed_rect(context, ec, range)
        .map(|rect| (rect, "start"))
        .or_else(|| range_rect(context, ec, range).map(|rect| (rect, "whole range")))
        .unwrap_or_else(|| (mouse_anchor(), "mouse"));
    let repaired = repair_degenerate(&mut rect);
    crate::com::log::log(&format!(
        "anchor from {source}: ({},{},{},{}){}",
        rect.left,
        rect.top,
        rect.right,
        rect.bottom,
        if repaired { " (fragment repaired)" } else { "" },
    ));
    to_cursor(rect)
}

/// The rectangle of the start of the composition: a clone collapsed to its start, so the anchor stays put while the
/// word grows. WeChat answers well for an empty range (a cursor point) and badly for a non-empty one.
fn collapsed_rect(context: &ITfContext, ec: u32, range: &ITfRange) -> Option<RECT> {
    let clone = unsafe { range.Clone() }.ok()?;
    unsafe { clone.Collapse(ec, TF_ANCHOR_START) }.ok()?;
    range_rect(context, ec, &clone)
}

/// The insertion point (the cursor, or the selection if there is one). Used when there is no composition range to measure.
pub(crate) fn caret_rect(context: &ITfContext, ec: u32) -> CursorRect {
    match selection_range(context, ec) {
        Some(range) => anchor_rect(context, ec, &range),
        None => to_cursor(mouse_anchor()),
    }
}

/// The current selection; `GetSelection` hands over ownership, the caller releases it.
fn selection_range(context: &ITfContext, ec: u32) -> Option<ITfRange> {
    let mut selection = [TF_SELECTION::default()];
    let mut fetched = 0u32;
    unsafe {
        context
            .GetSelection(ec, TF_DEFAULT_SELECTION, &mut selection, &mut fetched)
            .ok()?;
    }
    if fetched == 0 {
        return None;
    }
    unsafe { ManuallyDrop::take(&mut selection[0].range) }
}

/// The last normal line height measured (physical pixels), for [`repair_degenerate`].
static LAST_LINE_HEIGHT: AtomicI32 = AtomicI32::new(0);

/// What counts as a line height: less is a cursor fragment, more is probably the whole edit box.
const MIN_LINE_HEIGHT: i32 = 8;
const MAX_LINE_HEIGHT: i32 = 200;

/// A fragment gets the top of the line and the last normal line height, so the window stays on one side of the line and
/// its bottom never lands in the line being typed. Returns whether it was repaired.
fn repair_degenerate(rect: &mut RECT) -> bool {
    let height = rect.bottom - rect.top;
    if (MIN_LINE_HEIGHT..=MAX_LINE_HEIGHT).contains(&height) {
        LAST_LINE_HEIGHT.store(height, Ordering::Relaxed);
        return false;
    }
    let remembered = LAST_LINE_HEIGHT.load(Ordering::Relaxed);
    if height < MIN_LINE_HEIGHT && remembered >= MIN_LINE_HEIGHT {
        rect.bottom = rect.top + remembered;
        return true;
    }
    false
}

fn range_rect(context: &ITfContext, ec: u32, range: &ITfRange) -> Option<RECT> {
    let mut rect = RECT::default();
    let mut clipped = BOOL(0);
    unsafe {
        let view = context.GetActiveView().ok()?;
        view.GetTextExt(ec, range, &mut rect, &mut clipped).ok()?;
    }
    if rect.right <= rect.left && rect.bottom <= rect.top {
        return None;
    }
    // Applications without a real text store (some full screen games) "succeed" with a rectangle off the screen
    // (-1000,-1000 was seen in Dota 2). That would pin the window to the wrong corner, so it counts as no answer and the
    // mouse is used: following the mouse beats being off the screen.
    if !rect_on_real_monitor(&rect) {
        crate::com::log::log(&format!(
            "GetTextExt gave a rectangle off the screen ({},{},{},{}), using the mouse",
            rect.left, rect.top, rect.right, rect.bottom
        ));
        return None;
    }
    Some(rect)
}

/// Whether the centre of the rectangle is on a monitor. Negative coordinates of a second monitor are fine; only a
/// rectangle outside all of them is rejected.
fn rect_on_real_monitor(rect: &RECT) -> bool {
    let center = POINT {
        x: rect.left + (rect.right - rect.left) / 2,
        y: rect.top + (rect.bottom - rect.top) / 2,
    };
    unsafe { !MonitorFromPoint(center, MONITOR_DEFAULTTONULL).is_invalid() }
}

fn mouse_anchor() -> RECT {
    let mut point = POINT::default();
    let _ = unsafe { GetCursorPos(&mut point) };
    RECT {
        left: point.x,
        top: point.y,
        right: point.x,
        bottom: point.y + 16,
    }
}

fn to_cursor(rect: RECT) -> CursorRect {
    CursorRect {
        x: rect.left as f32,
        y: rect.top as f32,
        width: (rect.right - rect.left) as f32,
        height: (rect.bottom - rect.top) as f32,
        surface_relative: false,
    }
}
