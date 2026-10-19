//! What the plugin needs from the thing that shows the popup, so the Wayland part can be replaced by a fake in tests.
#pragma once
#include <cstdint>
#include <functional>
#include <fcitx/inputcontext.h>
namespace bornopata {
/// A bitmap the Server drew into a slot of a pool (see `PopupFrame` in the protocol).
struct Bitmap {
    uint64_t pool = 0;

    uint64_t buffer = 0;

    uint32_t offset = 0;

    uint32_t width = 0;

    uint32_t height = 0;

    uint32_t stride = 0;

    /// The scale the bitmap was drawn at, in 120ths.
    uint32_t scale120 = 120;

    /// The size the popup has on screen, in logical pixels.
    uint32_t logicalWidth = 0;

    uint32_t logicalHeight = 0;
};

class Presenter {
public:
    virtual ~Presenter() = default;

    /// Whether the popup can be shown for this context: a Wayland input-method context whose popup surface can be created.
    virtual bool available(fcitx::InputContext *context) = 0;

    /// A pool of shared memory the bitmaps live in; the presenter owns the descriptor from now on. A newer pool replaces the older ones.
    virtual void addPool(uint64_t pool, int fd, uint64_t size) = 0;

    /// Shows the bitmap in the popup of `context`. `false` when it cannot be shown; the buffer is then free again.
    virtual bool show(fcitx::InputContext *context, const Bitmap &bitmap) = 0;

    /// Takes the popup away. Every buffer the compositor still held counts as released.
    virtual void hide() = 0;

    /// Called with a buffer number when the Server may draw into it again.
    std::function<void(uint64_t)> onReleased;

    /// Called with the preferred scale of the popup in 120ths, once known and whenever it changes.
    std::function<void(uint32_t)> onScale;
};
}
