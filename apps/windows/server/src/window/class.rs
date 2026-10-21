//! A window class registered once per process.

use std::sync::Once;
use std::sync::atomic::{AtomicBool, Ordering};

use windows::Win32::Foundation::{E_FAIL, ERROR_CLASS_ALREADY_EXISTS, GetLastError};
use windows::Win32::UI::WindowsAndMessaging::{RegisterClassExW, WNDCLASSEXW};
use windows::core::{Error, Result};

/// Lives in a `static`; [`Self::ensure`] before each window is made.
pub(super) struct WindowClass {
    once: Once,

    /// The first registration failed, so every later one does too.
    failed: AtomicBool,
}

impl WindowClass {
    pub(super) const fn new() -> Self {
        Self {
            once: Once::new(),
            failed: AtomicBool::new(false),
        }
    }

    /// Registers the class built by `build` the first time (`cbSize` is filled in here). A class that exists already
    /// is no failure.
    pub(super) fn ensure(&self, build: impl FnOnce() -> WNDCLASSEXW) -> Result<()> {
        self.once.call_once(|| {
            let mut class = build();
            class.cbSize = size_of::<WNDCLASSEXW>() as u32;
            let atom = unsafe { RegisterClassExW(&class) };
            if atom == 0 && unsafe { GetLastError() } != ERROR_CLASS_ALREADY_EXISTS {
                self.failed.store(true, Ordering::Relaxed);
            }
        });
        if self.failed.load(Ordering::Relaxed) {
            Err(Error::from(E_FAIL))
        } else {
            Ok(())
        }
    }
}
