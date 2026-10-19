//! The real Rust Server with its popup render thread against the plugin's channel and a fake presenter: a bitmap per frame, scale changes and hiding on commit.
#include "bornopata.h"
#include "support/context.h"
#include "support/loop.h"
#include "support/presenter.h"
#include "support/real.h"
#include <fcitx/inputcontextmanager.h>
#include <fcitx/inputpanel.h>
int main(int argc, char **argv) {
    assert(argc == 2);
    RealServer server(argv[1], "", "popup");
    char program[] = "borno-test"; char disable[] = "--disable=all";
    char *arguments[] = {program, disable, nullptr};
    fcitx::Instance instance(2, arguments);
    instance.initialize();
    auto presenter = std::make_unique<FakePresenter>();
    auto *fake = presenter.get();
    {
        fcitx::BornopataEngine engine(&instance.addonManager(), std::move(presenter));
        Context context(instance.inputContextManager());
        context.setCapabilityFlags(fcitx::CapabilityFlag::Preedit);
        context.focusIn();
        auto press = [&](fcitx::KeySym sym) { return engine.process(&context, fcitx::Key(sym)); };
        auto waitFor = [&](auto done) {
            for (int i = 0; i < 100 && !done(); ++i) pump(instance, 50000);
            return done();
        };
        for (char c : std::string("ami")) assert(press(static_cast<fcitx::KeySym>(c)));
        if (!waitFor([&] { return fake->visible; })) {
            // Without a usable font the Server cannot draw; that is not what this test is about.
            assert(server.log().find("could not start") != std::string::npos);
            std::puts("skipped: the Server has no font for the popup");
            return 0;
        }
        // The Server draws the word: the Fcitx5 panel stays empty and only the inline Bangla preedit is the plugin's.
        assert(context.inputPanel().clientPreedit().toString() == "আমি");
        assert(!context.inputPanel().candidateList() && context.inputPanel().preedit().empty());
        const auto first = fake->shown.back();
        assert(first.context == FakePresenter::identity(&context));
        assert(first.bitmap.width > 100 && first.bitmap.height > 60 && first.bitmap.scale120 == 120);
        // The card is opaque in the middle of the bitmap (ARGB8888, little-endian: alpha is the fourth byte).
        uint8_t alpha = 0;
        const auto middle = first.bitmap.offset + (first.bitmap.height / 2) * first.bitmap.stride + (first.bitmap.width / 2) * 4 + 3;
        assert(pread(fake->pools.at(first.bitmap.pool).first, &alpha, 1, middle) == 1 && alpha == 255);

        // A new scale from the compositor draws the frame again, bigger, at the same logical size. The compositor gives
        // back every buffer it was shown before.
        pump(instance, 200000);
        for (const auto &shown : fake->shown) fake->onReleased(shown.bitmap.buffer);
        fake->onScale(240);
        assert(waitFor([&] { return fake->shown.back().bitmap.scale120 == 240; }));
        const auto &again = fake->shown.back().bitmap;
        assert(again.width > first.bitmap.width * 3 / 2);
        assert(again.logicalWidth <= first.bitmap.logicalWidth + 1 && again.logicalWidth + 1 >= first.bitmap.logicalWidth);

        // Space commits the word: the Server hides the popup.
        assert(press(FcitxKey_space));
        assert(waitFor([&] { return !fake->visible; }));
        assert(context.committed == "আমি ");
    }
    assert(server.stop());
}
