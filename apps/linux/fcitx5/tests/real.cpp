//! The real Rust Server with the default panel: typing, committing, editing, the Bangla/English toggle and privacy.
#include "bornopata.h"
#include "support/context.h"
#include "support/real.h"
#include <fcitx/inputcontextmanager.h>
#include <fcitx/inputpanel.h>
int main(int argc, char **argv) {
    assert(argc == 2);
    RealServer server(argv[1]);
    {
        char program[] = "borno-test"; char disable[] = "--disable=all";
        char *arguments[] = {program, disable, nullptr};
        fcitx::Instance instance(2, arguments);
        instance.initialize();
        fcitx::BornopataEngine engine(&instance.addonManager());
        Context context(instance.inputContextManager());
        context.setCapabilityFlags(fcitx::CapabilityFlag::Preedit);
        context.focusIn();
        auto press = [&](fcitx::KeySym sym, fcitx::KeyStates states = fcitx::KeyStates()) {
            return engine.process(&context, fcitx::Key(sym, states));
        };
        for (char c : std::string("ami")) assert(press(static_cast<fcitx::KeySym>(c)));
        assert(context.inputPanel().clientPreedit().toString() == "আমি");
        assert(context.inputPanel().preedit().toString() == "ami");
        auto list = context.inputPanel().candidateList();
        assert(list && list->size() > 0 && list->candidate(0).text().toString() == "আমি");
        // Space commits the word and a space.
        assert(press(FcitxKey_space));
        assert(context.committed == "আমি ");
        assert(context.inputPanel().clientPreedit().empty());
        // A stale click on the old list does nothing.
        list->candidate(0).select(&context);
        assert(context.committed == "আমি ");
        // Backspace edits the Roman buffer and passes through when it is empty.
        assert(press(FcitxKey_b) && press(FcitxKey_a));
        assert(press(FcitxKey_BackSpace));
        assert(context.inputPanel().preedit().toString().rfind("b", 0) == 0);
        assert(press(FcitxKey_BackSpace));
        assert(!press(FcitxKey_BackSpace));
        // Enter commits the word and then goes to the application.
        for (char c : std::string("ami")) assert(press(static_cast<fcitx::KeySym>(c)));
        assert(!press(FcitxKey_Return));
        assert(context.committed == "আমি আমি");
        // The toggle switches to English: letters now pass through. It switches back.
        const fcitx::KeyStates chord = fcitx::KeyStates(fcitx::KeyState::Ctrl) | fcitx::KeyState::Alt;
        assert(press(FcitxKey_space, chord));
        assert(!press(FcitxKey_a));
        assert(context.inputPanel().empty());
        assert(press(FcitxKey_space, chord));
        assert(press(FcitxKey_a));
        fcitx::InputMethodEntry entry("bornopata", "bornopata", "bn_BD", "bornopata");
        fcitx::InputContextEvent reset(&context, fcitx::EventType::InputContextReset);
        engine.reset(entry, reset);
        assert(context.inputPanel().empty());
        // Password fields are left alone.
        context.setCapabilityFlags(fcitx::CapabilityFlag::Password);
        assert(!press(FcitxKey_a));
        assert(context.inputPanel().empty());
    }
    assert(server.stop());
}
