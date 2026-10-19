//! A message that does not parse closes the channel and takes the popup away; the Server then draws nothing for this plugin until it attaches again.
#include "bridge.h"
#include <random>
#include <unistd.h>
namespace bornopata {
PopupBridge::PopupBridge(fcitx::EventLoop &loop, std::unique_ptr<Presenter> presenter, Lookup lookup)
    : loop_(loop), presenter_(std::move(presenter)), lookup_(std::move(lookup)) {
    std::random_device random;
    token_ = (uint64_t(random()) << 32 | random()) | 1;
    presenter_->onReleased = [this](uint64_t buffer) { release(buffer); };
    presenter_->onScale = [this](uint32_t scale) { channel_.send({{"Scale", {{"scale_120", scale}}}}); };
}

PopupBridge::~PopupBridge() {
    watcher_.reset();
    presenter_->onReleased = nullptr;
    presenter_->onScale = nullptr;
    presenter_->hide();
}

std::optional<uint64_t> PopupBridge::token(fcitx::InputContext *context, uint64_t generation) {
    if (!presenter_->available(context)) return std::nullopt;
    if (!channel_.connected() && attempted_ != generation) {
        attempted_ = generation;
        if (channel_.open(token_))
            watcher_ = loop_.addIOEvent(channel_.fd(), fcitx::IOEventFlag::In,
                [this](fcitx::EventSourceIO *, int, fcitx::IOEventFlags) { readable(); return true; });
    }
    if (!channel_.connected()) return std::nullopt;
    return token_;
}

void PopupBridge::hide(fcitx::InputContext *context) {
    if (!shownFor_.isNull() && shownFor_.get() == context) {
        shownFor_.unwatch();
        presenter_->hide();
    }
}

void PopupBridge::reset() {
    watcher_.reset();
    channel_.close();
    shownFor_.unwatch();
    presenter_->hide();
}

void PopupBridge::readable() {
    std::vector<Channel::Message> messages;
    const bool open = channel_.read(messages);
    for (auto &message : messages) handle(message);
    if (!open) reset();
}

void PopupBridge::handle(Channel::Message &message) {
    // The descriptor belongs to a Pool message and is the presenter's from here on, whatever happens to the message.
    int fd = std::exchange(message.fd, -1);
    try {
        const auto &body = message.body;
        if (body.contains("Pool")) {
            const auto &pool = body.at("Pool");
            presenter_->addPool(pool.at("pool").get<uint64_t>(), std::exchange(fd, -1), pool.at("size").get<uint64_t>());
        } else if (body.contains("Present")) {
            present(body.at("Present"));
        } else if (body == "Hide") {
            shownFor_.unwatch();
            presenter_->hide();
        }
    } catch (const nlohmann::json::exception &) {
        reset();
    }
    if (fd >= 0) ::close(fd);
}

void PopupBridge::present(const nlohmann::json &frame) {
    Bitmap bitmap;
    bitmap.pool = frame.at("pool").get<uint64_t>();
    bitmap.buffer = frame.at("buffer").get<uint64_t>();
    bitmap.offset = frame.at("offset").get<uint32_t>();
    bitmap.width = frame.at("width").get<uint32_t>();
    bitmap.height = frame.at("height").get<uint32_t>();
    bitmap.stride = frame.at("stride").get<uint32_t>();
    bitmap.scale120 = frame.at("scale_120").get<uint32_t>();
    bitmap.logicalWidth = frame.at("logical_width").get<uint32_t>();
    bitmap.logicalHeight = frame.at("logical_height").get<uint32_t>();
    auto *context = lookup_(frame.at("context").get<std::string>());
    // A frame for a context that lost the focus or stopped composing is stale: the Server hears that it is free.
    if (!context || !presenter_->show(context, bitmap)) {
        release(bitmap.buffer);
        return;
    }
    shownFor_ = context->watch();
}

void PopupBridge::release(uint64_t buffer) { channel_.send({{"Released", {{"buffer", buffer}}}}); }
}
