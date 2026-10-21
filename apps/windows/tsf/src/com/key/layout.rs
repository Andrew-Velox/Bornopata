//! Digits and punctuation as the current keyboard layout types them, looked up without disturbing dead key state.

use windows::Win32::UI::Input::KeyboardAndMouse::{
    GetKeyboardLayout, GetKeyboardState, HKL, MAPVK_VK_TO_VSC, MapVirtualKeyExW, ToUnicodeEx,
};

pub(super) fn character(vk: u32) -> Option<char> {
    // The main digits, the keypad (digits and operators), the OEM punctuation keys and Space. Other keys type nothing.
    if !matches!(vk, 0x30..=0x39 | 0x60..=0x6f | 0xba..=0xc0 | 0xdb..=0xde | 0x20) {
        return None;
    }
    let mut state = [0; 256];
    unsafe { GetKeyboardState(&mut state) }.ok()?;
    resolve(vk, &state, unsafe { GetKeyboardLayout(0) })
}

fn resolve(vk: u32, state: &[u8; 256], layout: HKL) -> Option<char> {
    let scan = unsafe { MapVirtualKeyExW(vk, MAPVK_VK_TO_VSC, Some(layout)) };
    let mut buffer = [0; 8];
    // Since Windows 10 1607 bit 2 keeps the lookup from changing the dead key state.
    let count = unsafe { ToUnicodeEx(vk, scan, state, &mut buffer, 1 << 2, Some(layout)) };
    // One UTF-16 unit only: a dead key, no mapping or a longer result produces no character.
    if count != 1 {
        return None;
    }
    char::from_u32(u32::from(buffer[0]))
}
