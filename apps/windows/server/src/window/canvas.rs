//! A top-down 32 bit DIB with a memory DC, the bitmap `UpdateLayeredWindow` takes.

use windows::Win32::Foundation::E_FAIL;
use windows::Win32::Graphics::Gdi::{
    BI_RGB, BITMAPINFO, BITMAPINFOHEADER, CreateCompatibleDC, CreateDIBSection, DIB_RGB_COLORS,
    DeleteDC, DeleteObject, HBITMAP, HDC, HGDIOBJ, SelectObject,
};
use windows::core::{Error, Result};

/// The pixels are reached only through [`Self::pixels`] (`&mut self`), so no Rust slice points at the memory while GDI
/// draws into it.
pub(super) struct Canvas {
    dib: HBITMAP,
    memdc: HDC,
    previous: HGDIOBJ,
    bits: *mut u8,
    len: usize,
}

impl Canvas {
    pub(super) fn new(width: i32, height: i32) -> Result<Self> {
        let header = BITMAPINFOHEADER {
            biSize: size_of::<BITMAPINFOHEADER>() as u32,
            biWidth: width,
            // A negative height makes the rows run top-down.
            biHeight: -height,
            biPlanes: 1,
            biBitCount: 32,
            biCompression: BI_RGB.0,
            ..Default::default()
        };
        let info = BITMAPINFO {
            bmiHeader: header,
            ..Default::default()
        };
        let mut bits = std::ptr::null_mut();
        let dib = unsafe { CreateDIBSection(None, &info, DIB_RGB_COLORS, &mut bits, None, 0)? };
        let (memdc, previous) = unsafe {
            let memdc = CreateCompatibleDC(None);
            (memdc, SelectObject(memdc, dib.into()))
        };
        if bits.is_null() || memdc.is_invalid() {
            unsafe {
                let _ = DeleteDC(memdc);
                let _ = DeleteObject(dib.into());
            }
            return Err(Error::from(E_FAIL));
        }
        Ok(Self {
            dib,
            memdc,
            previous,
            bits: bits.cast(),
            len: (width * height * 4) as usize,
        })
    }

    pub(super) fn dc(&self) -> HDC {
        self.memdc
    }

    /// The pixels, B, G, R, A, top-down.
    pub(super) fn pixels(&mut self) -> &mut [u8] {
        // SAFETY: `CreateDIBSection` allocated `len` contiguous bytes that live as long as `dib`, and `&mut self`
        // makes this the only access.
        unsafe { std::slice::from_raw_parts_mut(self.bits, self.len) }
    }
}

impl Drop for Canvas {
    fn drop(&mut self) {
        unsafe {
            SelectObject(self.memdc, self.previous);
            let _ = DeleteDC(self.memdc);
            let _ = DeleteObject(self.dib.into());
        }
    }
}
