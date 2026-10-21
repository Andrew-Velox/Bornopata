//! Sending a key again. When the Server settles the word being typed and lets the key through (Enter, an arrow, Ctrl+C),
//! the text has to be in the document before the application sees the key. The text is written by an edit session that may
//! run after the key has been answered, so the key is eaten, and sent once more when the text is in.

use std::cell::RefCell;
use std::collections::HashSet;

use windows::Win32::UI::Input::KeyboardAndMouse::{
    INPUT, INPUT_0, INPUT_KEYBOARD, KEYBD_EVENT_FLAGS, KEYBDINPUT, KEYEVENTF_EXTENDEDKEY,
    KEYEVENTF_KEYUP, SendInput, VIRTUAL_KEY,
};

thread_local! {
    /// Keys sent again and not yet seen coming back, so this service lets their press and release through once.
    static SENT: RefCell<HashSet<u32>> = RefCell::new(HashSet::new());
}

/// Presses and releases `vk` for the application of this thread.
pub(crate) fn send(vk: u32) {
    SENT.with(|sent| sent.borrow_mut().insert(vk));
    let flags = if is_extended(vk) {
        KEYEVENTF_EXTENDEDKEY
    } else {
        KEYBD_EVENT_FLAGS(0)
    };
    let key = |flags: KEYBD_EVENT_FLAGS| INPUT {
        r#type: INPUT_KEYBOARD,
        Anonymous: INPUT_0 {
            ki: KEYBDINPUT {
                wVk: VIRTUAL_KEY(vk as u16),
                dwFlags: flags,
                ..Default::default()
            },
        },
    };
    let inputs = [key(flags), key(flags | KEYEVENTF_KEYUP)];
    if unsafe { SendInput(&inputs, size_of::<INPUT>() as i32) } != inputs.len() as u32 {
        crate::com::log::log("could not send the key again");
        SENT.with(|sent| sent.borrow_mut().remove(&vk));
    }
}

/// Whether `vk` was sent again by [`send`] and has not come back yet; the first call says yes and forgets it.
pub(super) fn is_ours(vk: u32) -> bool {
    SENT.with(|sent| sent.borrow().contains(&vk))
}

/// The key came back and went through both its press and its release.
pub(super) fn arrived(vk: u32) {
    SENT.with(|sent| sent.borrow_mut().remove(&vk));
}

/// Arrows, Home, End, Page Up and Down, Insert and Delete are extended keys; without the flag a key sent for them acts
/// as the same key on the number pad.
fn is_extended(vk: u32) -> bool {
    matches!(vk, 0x21..=0x28 | 0x2d | 0x2e)
}
