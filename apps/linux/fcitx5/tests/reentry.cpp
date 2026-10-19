//! The real Server's reply commits before the redraw; a reentrant redraw must not undo the commit or let a consumed key through.
#include "bornopata.h"
#include "support/context.h"
#include "support/real.h"
#include <fcitx/inputcontextmanager.h>
#include <fcitx/inputpanel.h>
#include <stdexcept>
int main(int argc, char **argv) {
    assert(argc == 3);
    const std::string scenario = argv[2];
    RealServer server(argv[1]);
    {
        char program[] = "borno-test"; char disable[] = "--disable=all";
        char *arguments[] = {program, disable, nullptr};
        fcitx::Instance instance(2, arguments);
        instance.initialize();
        fcitx::BornopataEngine engine(&instance.addonManager());
        auto owned = std::make_unique<Context>(instance.inputContextManager());
        auto *context = owned.get();
        context->setCapabilityFlags(fcitx::CapabilityFlag::Preedit);
        context->focusIn();
        for (char c : std::string("ami")) assert(engine.process(context, fcitx::Key(static_cast<fcitx::KeySym>(c))));
        auto list = context->inputPanel().candidateList();
        assert(list && list->candidate(0).text().toString() == "আমি");
        bool armed = true;
        const auto eventType = scenario == "ui-reset" ? fcitx::EventType::InputContextUpdateUI : fcitx::EventType::InputContextUpdatePreedit;
        auto watcher = instance.watchEvent(eventType, fcitx::EventWatcherPhase::PreInputMethod, [&](fcitx::Event &) {
            if (!armed || scenario == "destroy") return;
            armed = false;
            if (scenario == "password" || scenario == "click") context->setCapabilityFlags(fcitx::CapabilityFlag::Password);
            else if (scenario == "sensitive") context->setCapabilityFlags(fcitx::CapabilityFlag::Sensitive);
            else if (scenario == "disable") context->setCapabilityFlags(fcitx::CapabilityFlag::Disable);
            else if (scenario == "reset" || scenario == "ui-reset") {
                fcitx::InputMethodEntry entry("bornopata", "bornopata", "bn_BD", "bornopata");
                fcitx::InputContextEvent reset(context, fcitx::EventType::InputContextReset);
                engine.reset(entry, reset);
            } else if (scenario == "focus") {
                context->focusOut();
                context->focusIn(); // hasFocus ends up true again; the lifecycle count must still reject the old redraw.
            } else if (scenario == "display-failure") throw std::runtime_error("test display failure");
            else assert(false);
        });
        // Destroy from the end of the frontend callback; Fcitx still holds raw pointers while it iterates events.
        if (scenario == "destroy") context->onPreedit = [&] { armed = false; owned.reset(); };
        if (scenario == "click") list->candidate(0).select(context);
        else assert(engine.process(context, fcitx::Key(FcitxKey_space)));
        assert(!armed);
        if (owned) {
            assert(context->committed == (scenario == "click" ? "আমি" : "আমি "));
            list->candidate(0).select(context);
            assert(context->committed == (scenario == "click" ? "আমি" : "আমি "));
        }
    }
    assert(server.stop());
}
