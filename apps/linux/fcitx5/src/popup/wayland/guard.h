//! The user data of the popup's `wl_surface`, which keeps Fcitx5 from crashing when the pointer or a finger meets it.
#pragma once
namespace bornopata {
/// Fcitx5's pointer and touch handlers turn every surface of its connection into its own wrapper object through the
/// user data of the surface's proxy, and call `userData()` on the result; a surface it did not make has no user data, so
/// they dereference a null pointer and Fcitx5 dies (and the keyboard with it, until it is restarted). The wrapper's
/// `userData()` reads a pointer-sized field a few words into the object. A block of zeros that is larger than that
/// object makes the handlers see "no window of mine" and return, which is what they do for a surface of another kind.
struct Guard {
    /// Stands for the wrapper object: all zero, larger than any layout Fcitx5's wrapper has had.
    alignas(16) unsigned char foreign[256] = {};

    /// What this plugin's own listeners need back from the proxy's user data.
    void *owner = nullptr;
};
}
