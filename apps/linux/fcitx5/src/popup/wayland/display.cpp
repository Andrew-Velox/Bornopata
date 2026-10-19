//! The registry may deliver the globals after construction; `ready()` says when they are all there.
#include "display.h"
#include <cstring>
namespace bornopata {
namespace {
void global(void *data, wl_registry *registry, uint32_t name, const char *interface, uint32_t version) {
    auto *self = static_cast<Display *>(data);
    auto is = [&](const wl_interface &wanted) { return std::strcmp(interface, wanted.name) == 0; };
    if (is(wl_compositor_interface)) {
        // Version 4 has damage_buffer, 6 tells the preferred buffer scale.
        self->compositorVersion = std::min<uint32_t>(version, 6);
        self->compositor = static_cast<wl_compositor *>(wl_registry_bind(registry, name, &wl_compositor_interface, self->compositorVersion));
    } else if (is(wl_shm_interface)) {
        self->shm = static_cast<wl_shm *>(wl_registry_bind(registry, name, &wl_shm_interface, 1));
    } else if (is(wp_viewporter_interface)) {
        self->viewporter = static_cast<wp_viewporter *>(wl_registry_bind(registry, name, &wp_viewporter_interface, 1));
    } else if (is(wp_fractional_scale_manager_v1_interface)) {
        self->fractional = static_cast<wp_fractional_scale_manager_v1 *>(
            wl_registry_bind(registry, name, &wp_fractional_scale_manager_v1_interface, 1));
    }
}
void globalRemove(void *, wl_registry *, uint32_t) {}
const wl_registry_listener LISTENER = {global, globalRemove};
}
Display::Display(std::string connection, wl_display *wl) : name("wayland:" + std::move(connection)), display(wl) {
    wl_registry_add_listener(wl_display_get_registry(wl), &LISTENER, this);
    wl_display_flush(wl);
}
}
