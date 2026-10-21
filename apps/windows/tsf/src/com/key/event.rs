//! A virtual key code from TSF to a protocol key event.

use borno_platform::protocol::{KeyEvent, KeyModifiers};
use windows::Win32::UI::Input::KeyboardAndMouse::{
    GetKeyState, VIRTUAL_KEY, VK_CAPITAL, VK_CONTROL, VK_LWIN, VK_MENU, VK_RWIN, VK_SHIFT,
};

/// The event for the key `vk`: the modifiers held now, and the character the key types in the current layout.
pub(crate) fn to_key_event(vk: u32) -> KeyEvent {
    let modifiers = current_modifiers();
    KeyEvent::new(
        vk,
        resolve_char(vk, modifiers.shift, modifiers.caps),
        modifiers,
    )
}

/// A modifier or lock key pressed alone. The Server must not see these: a lone Shift would otherwise commit the word
/// being typed.
pub(crate) fn is_modifier_key(vk: u32) -> bool {
    // Shift, Control, Alt (with the left/right variants), the Windows keys, Caps Lock, Num Lock, Scroll Lock.
    matches!(vk, 0x10..=0x12 | 0x14 | 0x5b | 0x5c | 0x90 | 0x91 | 0xa0..=0xa5)
}

/// A letter key: the virtual key codes of A to Z are their ASCII capitals.
fn is_letter(vk: u32) -> bool {
    (0x41..=0x5a).contains(&vk)
}

fn current_modifiers() -> KeyModifiers {
    KeyModifiers {
        ctrl: key_down(VK_CONTROL),
        shift: key_down(VK_SHIFT),
        alt: key_down(VK_MENU),
        win: key_down(VK_LWIN) || key_down(VK_RWIN),
        caps: key_toggled(VK_CAPITAL),
    }
}

/// The high bit of the key state is set (the value is negative) while the key is down.
fn key_down(vk: VIRTUAL_KEY) -> bool {
    unsafe { GetKeyState(i32::from(vk.0)) < 0 }
}

/// The low bit is set while a lock key is on.
fn key_toggled(vk: VIRTUAL_KEY) -> bool {
    unsafe { GetKeyState(i32::from(vk.0)) & 1 != 0 }
}

/// A letter's case is Shift xor Caps Lock; digits and punctuation follow the keyboard layout; function keys have none.
fn resolve_char(vk: u32, shift: bool, caps: bool) -> Option<char> {
    if is_letter(vk) {
        let lower = (b'a' + (vk - 0x41) as u8) as char;
        return Some(if shift != caps {
            lower.to_ascii_uppercase()
        } else {
            lower
        });
    }
    super::layout::character(vk)
}
