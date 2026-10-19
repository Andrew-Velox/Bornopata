//! Real framework event order: focus out delivers once, and the capabilities and Shift facts are forwarded in full.
#include "bornopata.h"
#include "support/context.h"
#include "support/mock.h"
#include <fcitx/inputcontextmanager.h>
#include <fcitx/inputpanel.h>
int main(int argc, char **argv) {
    assert(argc == 2);
    const std::string scenario = argv[1];
    bornopata::test::Mock mock("both");
    char program[] = "borno-test"; char disable[] = "--disable=all";
    char *arguments[] = {program, disable, nullptr};
    fcitx::Instance instance(2, arguments);
    instance.initialize();
    fcitx::BornopataEngine engine(&instance.addonManager());
    fcitx::InputMethodEntry entry("bornopata", "bornopata", "bn_BD", "bornopata");
    auto context = std::make_unique<Context>(instance.inputContextManager());
    using Flag = fcitx::CapabilityFlag;
    fcitx::CapabilityFlags capabilities = Flag::Preedit;
    if (scenario == "focusout-panel") capabilities = Flag::SurroundingText;
    if (scenario == "focusout-client-owned") capabilities |= Flag::ClientUnfocusCommit;
    context->setCapabilityFlags(capabilities);
    context->focusIn();
    assert(engine.process(context.get(), fcitx::Key(FcitxKey_a)));
    if (scenario.rfind("focusout-", 0) == 0) {
        if (scenario == "focusout-client-owned") context->commitString(context->inputPanel().clientPreedit().toStringForCommit());
        context->focusOut();
        fcitx::FocusOutEvent event(context.get());
        engine.deactivate(entry, event);
        assert(context->committed == "\u0986\u09ae\u09bf");
    } else if (scenario == "capability-deactivate") {
        fcitx::InputContextSwitchInputMethodEvent event(fcitx::InputMethodSwitchedReason::CapabilityChanged, "bornopata", context.get());
        engine.deactivate(entry, event);
        assert(context->committed.empty());
    } else {
        context->setCapabilityFlags(scenario == "password" ? fcitx::CapabilityFlags(Flag::Password) :
            scenario == "sensitive" ? fcitx::CapabilityFlags(Flag::Sensitive) :
            scenario == "disabled" ? fcitx::CapabilityFlags(Flag::Disable) : fcitx::CapabilityFlags());
        if (scenario != "empty" && scenario != "shift-empty") assert(context->inputPanel().empty());
        if (scenario == "shift-empty") {
            fcitx::KeyEvent press(context.get(), fcitx::Key(FcitxKey_Shift_L), false);
            fcitx::KeyEvent release(context.get(), fcitx::Key(FcitxKey_Shift_L), true);
            engine.keyEvent(entry, press);
            engine.keyEvent(entry, release);
        }
    }
    context.reset();
    auto timer = instance.eventLoop().addTimeEvent(CLOCK_MONOTONIC, fcitx::now(CLOCK_MONOTONIC) + 30000, 0,
        [&](fcitx::EventSourceTime *, uint64_t) { instance.eventLoop().exit(); return false; });
    instance.eventLoop().exec();
    mock.join();
    bool sawShiftPress = false, sawShiftRelease = false, sawCapabilityReason = false;
    Json lastCaps;
    for (const auto &message : mock.messages) {
        if (!message.contains("Event")) continue;
        const auto &event = message.at("Event").at("event");
        if (event.contains("Capabilities")) lastCaps = event.at("Capabilities");
        if (event.contains("Key") && event.at("Key").at("event").at("virtual_key") == 16) {
            if (event.at("Key").at("release").get<bool>()) sawShiftRelease = true;
            else sawShiftPress = true;
        }
        if (event.contains("Deactivate")) sawCapabilityReason |= event.at("Deactivate").at("capability_changed").get<bool>();
    }
    assert(lastCaps.at("sensitive") == (scenario == "sensitive"));
    assert(lastCaps.at("password") == (scenario == "password"));
    assert(lastCaps.at("disabled") == (scenario == "disabled"));
    if (scenario == "shift-empty") assert(sawShiftPress && sawShiftRelease);
    if (scenario == "capability-deactivate") assert(sawCapabilityReason);
}
