//! The popup channel and presenter against the mock Server: attaching, showing what is pushed, reporting releases and scale, and the stale and lost cases.
#include "bornopata.h"
#include "support/context.h"
#include "support/loop.h"
#include "support/mock.h"
#include "support/presenter.h"
#include <fcitx/inputcontextmanager.h>
#include <fcitx/inputpanel.h>
#include <sys/socket.h>

namespace {
bool has(bornopata::test::Mock &mock, const std::string &name, const std::string &key, uint64_t value) {
    std::lock_guard guard(mock.lock);
    for (const auto &message : mock.channelMessages)
        if (message.contains(name) && message.at(name).at(key) == value) return true;
    return false;
}
}

int main(int argc, char **argv) {
    assert(argc == 2);
    const std::string scenario = argv[1];
    bornopata::test::Mock mock("popup");
    char program[] = "borno-test"; char disable[] = "--disable=all";
    char *arguments[] = {program, disable, nullptr};
    fcitx::Instance instance(2, arguments);
    instance.initialize();
    auto presenter = std::make_unique<FakePresenter>();
    auto *fake = presenter.get();
    fake->canShow = scenario != "unavailable";
    {
        fcitx::BornopataEngine engine(&instance.addonManager(), std::move(presenter));
        auto owned = std::make_unique<Context>(instance.inputContextManager());
        auto &context = *owned;
        context.setCapabilityFlags(fcitx::CapabilityFlag::Preedit);
        context.focusIn();
        assert(engine.process(&context, fcitx::Key(FcitxKey_a)));
        pump(instance);
        if (scenario == "unavailable") {
            // The presenter cannot show the popup for this context: no channel, no token, the Server is not asked to draw there.
            assert(mock.attached == 0 && mock.opens.at(0).at("popup").is_null());
            assert(fake->shown.empty());
        } else {
            assert(mock.attached != 0 && mock.opens.at(0).at("popup") == mock.attached.load());
            // The frame is the Server's: the Fcitx5 panel stays empty, and the bitmap it pushed is in the popup.
            assert(!context.inputPanel().candidateList() && context.inputPanel().preedit().empty());
            assert(fake->pools.size() == 1 && fake->shown.size() == 1 && fake->visible);
            const auto &first = fake->shown.at(0);
            assert(first.context == mock.opens.at(0).at("context").get<std::string>());
            assert(first.first == 0x7f && first.bitmap.width == 8 && first.bitmap.height == 4 && first.bitmap.stride == 32);
            assert(first.bitmap.buffer == 1 && first.bitmap.logicalWidth == 8 && first.bitmap.scale120 == 120);
        }
        if (scenario == "release") {
            // What the presenter learns from the compositor goes back to the Server.
            fake->onReleased(1);
            fake->onScale(150);
            pump(instance);
            assert(has(mock, "Released", "buffer", 1) && has(mock, "Scale", "scale_120", 150));
        }
        if (scenario == "stale") {
            // The focus leaves: the popup goes at once; the bitmap that was already on its way is refused and given back.
            const int before = fake->hides;
            context.focusOut();
            assert(!fake->visible && fake->hides > before);
            pump(instance);
            assert(fake->shown.size() == 1 && !fake->visible);
            assert(has(mock, "Released", "buffer", 2));
        }
        if (scenario == "hangup") {
            shutdown(mock.channel, SHUT_RDWR);
            pump(instance);
            assert(!fake->visible && fake->hides > 0);
            // The session connection still works; the Server just draws nothing in the popup any more.
            assert(engine.process(&context, fcitx::Key(FcitxKey_a)));
        }
        owned.reset();
        pump(instance, 30000);
    }
    pump(instance, 30000);
    mock.join();
}
