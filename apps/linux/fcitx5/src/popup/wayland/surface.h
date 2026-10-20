//! The popup surface of one input context: the `wl_surface` the compositor places at the caret, and what shows a buffer on it at the right size.
#pragma once
#include "display.h"
#include "guard.h"
#include "popup/presenter.h"
#include <functional>
#include <memory>
extern "C" {
#include "input-method-unstable-v2-client-protocol.h"
}
namespace bornopata {
class Surface {
public:
    /// Gives `input` the popup role on a new surface. `onScale` hears the preferred scale (in 120ths) as the compositor tells it. `nullptr` when the surface cannot be made.
    static std::unique_ptr<Surface> create(Display &display, zwp_input_method_v2 *input, std::function<void(uint32_t)> onScale);
    Surface(const Surface &) = delete;
    Surface &operator=(const Surface &) = delete;
    ~Surface();

    /// Shows `buffer` (a `bitmap` of the pool) at its logical size.
    void attach(wl_buffer *buffer, const Bitmap &bitmap);

    /// The connection is gone: forget the objects instead of destroying them.
    void abandon();

    Display &display() { return display_; }
private:
    Surface(Display &display, std::function<void(uint32_t)> onScale);

    static void preferredBufferScale(void *data, wl_surface *, int32_t factor);
    static void preferredScale(void *data, wp_fractional_scale_v1 *, uint32_t scale);

    Display &display_;
    std::function<void(uint32_t)> onScale_;

    /// The surface's user data; see guard.h. It outlives the proxy.
    Guard guard_;
    wl_surface *surface_ = nullptr;
    zwp_input_popup_surface_v2 *popup_ = nullptr;
    wp_viewport *viewport_ = nullptr;
    wp_fractional_scale_v1 *fractional_ = nullptr;

    /// The compositor sends the fractional scale or, without it, the integer one; only one of them is listened to.
    bool fractionalScale_ = false;
};
}
