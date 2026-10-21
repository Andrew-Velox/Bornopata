//! Which monitor the text cursor is on: its work area and its scale.

use borno_window::Rect;
use windows::Win32::Foundation::POINT;
use windows::Win32::Graphics::Gdi::{
    GetMonitorInfoW, MONITOR_DEFAULTTONEAREST, MONITORINFO, MonitorFromPoint,
};
use windows::Win32::UI::HiDpi::{GetDpiForMonitor, MDT_EFFECTIVE_DPI};

/// What the window needs to know about the screen at a point, in physical pixels.
pub(super) struct Screen {
    /// The work area (the monitor without the taskbar) of the monitor nearest the point.
    pub(super) work: Rect,

    /// The monitor's scale: 1.0 at 96 dpi, 2.0 at 192 dpi.
    pub(super) scale: f32,
}

/// The screen at `point`. A monitor that cannot be asked gives a huge work area and scale 1, so the window is at least
/// not pushed out of sight.
pub(super) fn screen_at(point: POINT) -> Screen {
    let monitor = unsafe { MonitorFromPoint(point, MONITOR_DEFAULTTONEAREST) };
    let mut info = MONITORINFO {
        cbSize: size_of::<MONITORINFO>() as u32,
        ..Default::default()
    };
    let work = if unsafe { GetMonitorInfoW(monitor, &mut info) }.as_bool() {
        let area = info.rcWork;
        Rect::new(
            area.left as f32,
            area.top as f32,
            (area.right - area.left) as f32,
            (area.bottom - area.top) as f32,
        )
    } else {
        Rect::new(0.0, 0.0, 100_000.0, 100_000.0)
    };
    let (mut dpi_x, mut dpi_y) = (0, 0);
    let dpi = unsafe { GetDpiForMonitor(monitor, MDT_EFFECTIVE_DPI, &mut dpi_x, &mut dpi_y) }
        .map_or(96, |()| dpi_x.max(96));
    Screen {
        work,
        scale: dpi as f32 / 96.0,
    }
}
