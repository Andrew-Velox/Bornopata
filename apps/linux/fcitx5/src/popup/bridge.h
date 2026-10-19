//! Connects the popup channel to the presenter: attaches on demand, shows what the Server pushes and reports back.
#pragma once
#include "channel.h"
#include "presenter.h"
#include <memory>
#include <optional>
#include <fcitx-utils/event.h>
#include <fcitx/inputcontext.h>
namespace bornopata {
class PopupBridge {
public:
    /// Finds the input context with this identity, only when it is focused and showing a frame the Server draws; `nullptr` otherwise.
    using Lookup = std::function<fcitx::InputContext *(const std::string &)>;

    PopupBridge(fcitx::EventLoop &loop, std::unique_ptr<Presenter> presenter, Lookup lookup);
    PopupBridge(const PopupBridge &) = delete;
    PopupBridge &operator=(const PopupBridge &) = delete;
    ~PopupBridge();

    /// The token to name in `Open` when `context` can show the popup. The channel is attached on the first call of a Server connection `generation`; a failed attempt is not repeated within it.
    std::optional<uint64_t> token(fcitx::InputContext *context, uint64_t generation);

    /// Takes the popup away when it shows `context`.
    void hide(fcitx::InputContext *context);

    /// The Server connection ended: the channel goes with it.
    void reset();

    bool connected() const { return channel_.connected(); }
private:
    void readable();
    void handle(Channel::Message &message);
    void present(const nlohmann::json &frame);
    void release(uint64_t buffer);

    fcitx::EventLoop &loop_;
    std::unique_ptr<Presenter> presenter_;
    Lookup lookup_;
    Channel channel_;
    std::unique_ptr<fcitx::EventSourceIO> watcher_;
    fcitx::TrackableObjectReference<fcitx::InputContext> shownFor_;

    /// Names the channel in the Server; random per plugin instance.
    uint64_t token_;

    uint64_t attempted_ = 0;
};
}
