//! Requests are flushed right after they are made: Fcitx5's loop flushes only after it dispatched something.
#include "surface.h"
#include <algorithm>
#include <climits>
namespace bornopata {
namespace {
void noop(void *, wl_surface *, wl_output *) {}
void noopTransform(void *, wl_surface *, uint32_t) {}
}

Surface::Surface(Display &display, std::function<void(uint32_t)> onScale) : display_(display), onScale_(std::move(onScale)) {}

std::unique_ptr<Surface> Surface::create(Display &display, zwp_input_method_v2 *input, std::function<void(uint32_t)> onScale) {
    if (!display.ready() || !input) return nullptr;
    std::unique_ptr<Surface> self(new Surface(display, std::move(onScale)));
    self->surface_ = wl_compositor_create_surface(display.compositor);
    if (!self->surface_) return nullptr;
    // Its pointer and touch events are not wanted: the popup shows words, the keys choose them.
    if (auto *region = wl_compositor_create_region(display.compositor)) {
        wl_surface_set_input_region(self->surface_, region);
        wl_region_destroy(region);
    }
    self->fractionalScale_ = display.fractional && display.viewporter;
    if (self->fractionalScale_) {
        self->fractional_ = wp_fractional_scale_manager_v1_get_fractional_scale(display.fractional, self->surface_);
        static const wp_fractional_scale_v1_listener fractional = {preferredScale};
        wp_fractional_scale_v1_add_listener(self->fractional_, &fractional, self.get());
    } else if (display.compositorVersion >= 6) {
        static const wl_surface_listener listener = {noop, noop, preferredBufferScale, noopTransform};
        wl_surface_add_listener(self->surface_, &listener, self.get());
    }
    if (display.viewporter) self->viewport_ = wp_viewporter_get_viewport(display.viewporter, self->surface_);
    self->popup_ = zwp_input_method_v2_get_input_popup_surface(input, self->surface_);
    wl_display_flush(display.display);
    return self;
}

Surface::~Surface() {
    if (fractional_) wp_fractional_scale_v1_destroy(fractional_);
    if (viewport_) wp_viewport_destroy(viewport_);
    if (popup_) zwp_input_popup_surface_v2_destroy(popup_);
    if (surface_) wl_surface_destroy(surface_);
    if (surface_) wl_display_flush(display_.display);
}

void Surface::abandon() { fractional_ = nullptr; viewport_ = nullptr; popup_ = nullptr; surface_ = nullptr; }

void Surface::attach(wl_buffer *buffer, const Bitmap &bitmap) {
    if (viewport_) {
        // The bitmap is drawn at the compositor's scale; the viewport maps it onto the logical size the popup has.
        wp_viewport_set_destination(viewport_, static_cast<int32_t>(bitmap.logicalWidth), static_cast<int32_t>(bitmap.logicalHeight));
        wl_surface_set_buffer_scale(surface_, 1);
    } else {
        wl_surface_set_buffer_scale(surface_, std::max<int32_t>(1, static_cast<int32_t>(bitmap.scale120 / 120)));
    }
    wl_surface_attach(surface_, buffer, 0, 0);
    if (display_.compositorVersion >= 4) wl_surface_damage_buffer(surface_, 0, 0, static_cast<int32_t>(bitmap.width), static_cast<int32_t>(bitmap.height));
    else wl_surface_damage(surface_, 0, 0, INT32_MAX, INT32_MAX);
    wl_surface_commit(surface_);
    wl_display_flush(display_.display);
}

void Surface::preferredBufferScale(void *data, wl_surface *, int32_t factor) {
    auto *self = static_cast<Surface *>(data);
    if (self->onScale_ && factor > 0) self->onScale_(static_cast<uint32_t>(factor) * 120);
}

void Surface::preferredScale(void *data, wp_fractional_scale_v1 *, uint32_t scale) {
    auto *self = static_cast<Surface *>(data);
    if (self->onScale_) self->onScale_(scale);
}
}
