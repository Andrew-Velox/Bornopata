//! When a capability change clears the UI reentrantly, the real Server's learning must follow the latest privacy facts.
#include "bornopata.h"
#include "support/context.h"
#include "support/real.h"
#include <fcitx/inputcontextmanager.h>
#include <fcitx/inputpanel.h>
int main(int argc, char **argv) {
    assert(argc == 3);
    const std::string scenario = argv[2];
    RealServer server(argv[1], "[general]\nlearning = true\n");
    {
        char program[] = "borno-test"; char disable[] = "--disable=all";
        char *arguments[] = {program, disable, nullptr};
        fcitx::Instance instance(2, arguments);
        instance.initialize();
        fcitx::BornopataEngine engine(&instance.addonManager());
        Context context(instance.inputContextManager());
        using Flag = fcitx::CapabilityFlag;
        using Flags = fcitx::CapabilityFlags;
        context.setCapabilityFlags(Flag::Preedit);
        context.focusIn();
        auto type = [&](const std::string &text, bool consumed = true) {
            for (char c : text) assert(engine.process(&context, fcitx::Key(static_cast<fcitx::KeySym>(c))) == consumed);
        };
        type("ami ");
        assert(context.committed == "আমি ");
        type("kotha"); // A word typed before the capability change must be dropped at the boundary.
        bool armed = true;
        const bool blocked = scenario == "password-reset" || scenario == "disable-focus"
            || scenario == "superseded-password" || scenario == "superseded-disable";
        auto watcher = instance.watchEvent(fcitx::EventType::InputContextUpdatePreedit, fcitx::EventWatcherPhase::PreInputMethod, [&](fcitx::Event &) {
            if (!armed) return;
            armed = false;
            if (scenario == "sensitive-reset" || scenario == "password-reset") {
                fcitx::InputMethodEntry entry("bornopata", "bornopata", "bn_BD", "bornopata");
                fcitx::InputContextEvent reset(&context, fcitx::EventType::InputContextReset);
                engine.reset(entry, reset);
            } else if (scenario == "sensitive-focus" || scenario == "disable-focus") {
                context.focusOut();
                context.focusIn();
            } else if (scenario == "superseded-password") context.setCapabilityFlags(Flags(Flag::Preedit) | Flag::Sensitive | Flag::Password);
            else if (scenario == "superseded-disable") context.setCapabilityFlags(Flags(Flag::Preedit) | Flag::Sensitive | Flag::Disable);
            else if (scenario == "superseded-sensitive") context.setCapabilityFlags(Flags(Flag::Preedit) | Flag::Sensitive);
            else assert(scenario == "sensitive-key");
            // Type a key reentrantly before clear() returns: the Server must not reuse its old ordinary state.
            type("kotha ", !blocked);
        });
        context.setCapabilityFlags(Flags(Flag::Preedit) |
            (scenario == "password-reset" || scenario == "superseded-sensitive" ? Flag::Password :
             scenario == "disable-focus" ? Flag::Disable : Flag::Sensitive));
        assert(!armed);
        type("kotha ", !blocked); // After clearing returns, the cache must agree with the Server.
        assert(context.committed == (blocked ? "আমি " : "আমি কথা কথা "));
        context.setCapabilityFlags(Flag::Preedit);
        type("bangla ");
        assert(context.committed.ends_with("বাংলা "));
    }
    assert(server.stop());
    const auto learned = server.read("bornopata/user-words.tsv");
    assert(learned.find("আমি") != std::string::npos);
    assert(learned.find("বাংলা") != std::string::npos);
    assert(learned.find("কথা") == std::string::npos);
}
