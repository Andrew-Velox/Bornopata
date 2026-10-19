//! Shows the popup through the Wayland connection Fcitx5 already has, on the input method's own popup surface.
#pragma once
#include "display.h"
#include "popup/presenter.h"
#include "surface.h"
#include <map>
#include <memory>
#include <unordered_map>
#include <fcitx/addonmanager.h>
#include <fcitx-utils/handlertable.h>
#include <wayland_public.h>
namespace bornopata {
class WaylandPresenter final : public Presenter {
public:
    explicit WaylandPresenter(fcitx::AddonManager *manager);
    ~WaylandPresenter() override;

    bool available(fcitx::InputContext *context) override;
    void addPool(uint64_t pool, int fd, uint64_t size) override;
    bool show(fcitx::InputContext *context, const Bitmap &bitmap) override;
    void hide() override;
private:
    struct Pool {
        int fd = -1;

        uint64_t size = 0;

        wl_shm_pool *object = nullptr;
    };

    Display *displayOf(fcitx::InputContext *context);
    zwp_input_method_v2 *inputMethod(fcitx::InputContext *context);
    void retirePools(uint64_t keep);
public:
    /// The compositor released the buffer.
    void release(wl_buffer *buffer);
private:

    fcitx::AddonManager *manager_;
    std::unique_ptr<fcitx::HandlerTableEntry<fcitx::WaylandConnectionCreated>> created_;
    std::unique_ptr<fcitx::HandlerTableEntry<fcitx::WaylandConnectionClosed>> closed_;
    std::map<std::string, std::unique_ptr<Display>> displays_;
    std::map<uint64_t, Pool> pools_;
    std::unique_ptr<Surface> surface_;
    fcitx::TrackableObjectReference<fcitx::InputContext> shownFor_;

    /// Buffers the compositor may still read, with the Server's number for each.
    std::unordered_map<wl_buffer *, uint64_t> outstanding_;
};
}
