//! One Wayland connection of Fcitx5 and the globals the popup needs from it.
#pragma once
#include <cstdint>
#include <string>
#include <wayland-client.h>
extern "C" {
#include "fractional-scale-v1-client-protocol.h"
#include "viewporter-client-protocol.h"
}
namespace bornopata {
/// The proxies are created on Fcitx5's own event queue, so Fcitx5's loop delivers their events. They are never destroyed one by one: they go when the connection does.
struct Display {
    Display(std::string name, wl_display *display);
    Display(const Display &) = delete;
    Display &operator=(const Display &) = delete;

    /// The input context's `display()` of this connection, "wayland:" and the connection name.
    std::string name;

    wl_display *display;

    wl_compositor *compositor = nullptr;

    uint32_t compositorVersion = 0;

    wl_shm *shm = nullptr;

    wp_viewporter *viewporter = nullptr;

    wp_fractional_scale_manager_v1 *fractional = nullptr;

    /// What every popup needs is bound.
    bool ready() const { return compositor && shm; }
};
}
