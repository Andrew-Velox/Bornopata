//! Putting the renderer's bitmap on a layered window.

use borno_render::Pixmap;
use windows::Win32::Foundation::{COLORREF, E_INVALIDARG, HWND, POINT, SIZE};
use windows::Win32::Graphics::Gdi::{AC_SRC_ALPHA, AC_SRC_OVER, BLENDFUNCTION};
use windows::Win32::UI::WindowsAndMessaging::{ULW_ALPHA, UpdateLayeredWindow};
use windows::core::{Error, Result};

use super::canvas::Canvas;

/// Pastes `pixmap` (premultiplied RGBA, the shadow margin included) onto the layered window `hwnd` and moves the window
/// so that the bitmap's top-left corner is at `position` on the screen.
pub(super) fn present(hwnd: HWND, pixmap: &Pixmap, position: (i32, i32)) -> Result<()> {
    let (width, height) = (pixmap.width() as i32, pixmap.height() as i32);
    if width <= 0 || height <= 0 {
        return Err(Error::from(E_INVALIDARG));
    }
    let mut canvas = Canvas::new(width, height)?;
    // tiny-skia draws RGBA and a DIB is BGRA; both are premultiplied, so only the channels change places.
    for (target, source) in canvas.pixels().chunks_exact_mut(4).zip(pixmap.pixels()) {
        target[0] = source.blue();
        target[1] = source.green();
        target[2] = source.red();
        target[3] = source.alpha();
    }
    let blend = BLENDFUNCTION {
        BlendOp: AC_SRC_OVER as u8,
        BlendFlags: 0,
        SourceConstantAlpha: 255,
        AlphaFormat: AC_SRC_ALPHA as u8,
    };
    unsafe {
        UpdateLayeredWindow(
            hwnd,
            None,
            Some(&POINT {
                x: position.0,
                y: position.1,
            }),
            Some(&SIZE {
                cx: width,
                cy: height,
            }),
            Some(canvas.dc()),
            Some(&POINT { x: 0, y: 0 }),
            COLORREF(0),
            Some(&blend),
            ULW_ALPHA,
        )
    }
}
