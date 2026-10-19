//! Everything here runs on Fcitx5's main thread: the registry, buffer and scale events arrive in its event loop.
#include "presenter.h"
#include "make.h"
#include "proxy.h"
#include <fcitx/addoninstance.h>
#include <fcitx-utils/metastring.h>
#include <unistd.h>

// The signature of the function the Wayland frontend exports, as in Fcitx5's `waylandim_public.h`, which is not installed. The pointer it returns is an object of Fcitx5's wrapper classes, of which only the raw proxy inside is used.
namespace fcitx::wayland { class ZwpInputMethodV2; }
FCITX_ADDON_DECLARE_FUNCTION(WaylandIMModule, getInputMethodV2, fcitx::wayland::ZwpInputMethodV2 *(fcitx::InputContext *));

namespace bornopata {
namespace {
const wl_buffer_listener BUFFER = {[](void *data, wl_buffer *buffer) { static_cast<WaylandPresenter *>(data)->release(buffer); }};
}

std::unique_ptr<Presenter> makeWaylandPresenter(fcitx::AddonManager *manager) {
    if (!manager->addon("wayland", true)) return nullptr;
    return std::make_unique<WaylandPresenter>(manager);
}

WaylandPresenter::WaylandPresenter(fcitx::AddonManager *manager) : manager_(manager) {
    auto *wayland = manager->addon("wayland", true);
    created_ = wayland->call<fcitx::IWaylandModule::addConnectionCreatedCallback>(
        [this](const std::string &name, wl_display *display, fcitx::FocusGroup *) {
            auto entry = std::make_unique<Display>(name, display);
            displays_[entry->name] = std::move(entry);
        });
    closed_ = wayland->call<fcitx::IWaylandModule::addConnectionClosedCallback>(
        [this](const std::string &name, wl_display *) {
            auto found = displays_.find("wayland:" + name);
            if (found == displays_.end()) return;
            // The connection is going away: its objects go with it, so no request may be sent for them.
            if (surface_ && &surface_->display() == found->second.get()) {
                surface_->abandon();
                surface_.reset();
                shownFor_.unwatch();
                outstanding_.clear();
                for (auto &entry : pools_) entry.second.object = nullptr;
            }
            displays_.erase(found);
        });
}

WaylandPresenter::~WaylandPresenter() {
    hide();
    retirePools(UINT64_MAX);
}

Display *WaylandPresenter::displayOf(fcitx::InputContext *context) {
    auto found = displays_.find(context->display());
    return found == displays_.end() || !found->second->ready() ? nullptr : found->second.get();
}

zwp_input_method_v2 *WaylandPresenter::inputMethod(fcitx::InputContext *context) {
    if (context->frontendName() != "wayland_v2") return nullptr;
    auto *waylandim = manager_->addon("waylandim");
    if (!waylandim) return nullptr;
    auto *wrapper = waylandim->call<fcitx::IWaylandIMModule::getInputMethodV2>(context);
    if (!wrapper) return nullptr;
    return reinterpret_cast<zwp_input_method_v2 *>(findProxy(wrapper, "zwp_input_method_v2"));
}

bool WaylandPresenter::available(fcitx::InputContext *context) {
    return displayOf(context) && inputMethod(context);
}

void WaylandPresenter::addPool(uint64_t pool, int fd, uint64_t size) {
    retirePools(pool);
    pools_[pool] = Pool{fd, size, nullptr};
}

/// Pools older than `keep` are no longer used; their buffers stay valid in the compositor until it is done with them.
void WaylandPresenter::retirePools(uint64_t keep) {
    for (auto it = pools_.begin(); it != pools_.end();) {
        if (it->first >= keep) { ++it; continue; }
        if (it->second.object) wl_shm_pool_destroy(it->second.object);
        if (it->second.fd >= 0) ::close(it->second.fd);
        it = pools_.erase(it);
    }
}

bool WaylandPresenter::show(fcitx::InputContext *context, const Bitmap &bitmap) {
    auto *display = displayOf(context);
    auto pool = pools_.find(bitmap.pool);
    if (!display || pool == pools_.end()) return false;
    const uint64_t end = uint64_t(bitmap.offset) + uint64_t(bitmap.stride) * bitmap.height;
    if (bitmap.width == 0 || bitmap.height == 0 || bitmap.stride < bitmap.width * 4 || end > pool->second.size) return false;
    if (!surface_ || shownFor_.get() != context) {
        hide();
        auto *input = inputMethod(context);
        surface_ = Surface::create(*display, input, [this](uint32_t scale) { if (onScale) onScale(scale); });
        if (!surface_) return false;
        shownFor_ = context->watch();
    }
    if (!pool->second.object) pool->second.object = wl_shm_create_pool(display->shm, pool->second.fd, static_cast<int32_t>(pool->second.size));
    auto *buffer = wl_shm_pool_create_buffer(pool->second.object, static_cast<int32_t>(bitmap.offset), static_cast<int32_t>(bitmap.width),
        static_cast<int32_t>(bitmap.height), static_cast<int32_t>(bitmap.stride), WL_SHM_FORMAT_ARGB8888);
    if (!buffer) return false;
    wl_buffer_add_listener(buffer, &BUFFER, this);
    outstanding_[buffer] = bitmap.buffer;
    surface_->attach(buffer, bitmap);
    return true;
}

void WaylandPresenter::release(wl_buffer *buffer) {
    auto found = outstanding_.find(buffer);
    if (found == outstanding_.end()) return;
    const auto number = found->second;
    outstanding_.erase(found);
    wl_buffer_destroy(buffer);
    if (onReleased) onReleased(number);
}

void WaylandPresenter::hide() {
    shownFor_.unwatch();
    surface_.reset();
    // The compositor may never release a buffer of a surface that is gone; the Server must not wait for it.
    auto buffers = std::move(outstanding_);
    outstanding_.clear();
    for (auto &[buffer, number] : buffers) {
        wl_buffer_destroy(buffer);
        if (onReleased) onReleased(number);
    }
}
}
