//! Loading without a session bus, the default panel, candidate clicks and protocol failures.
#include "bornopata.h"
#include "key/mapping.h"
#include "support/context.h"
#include "support/mock.h"
#include <fcitx/inputcontextmanager.h>
#include <fcitx/inputpanel.h>
int main(int argc, char **argv) {
    assert(argc == 2);
    const std::string mode = argv[1];
    bornopata::test::Mock mock(mode);
    auto escaped = bornopata::mapKey(fcitx::Key(FcitxKey_quotedbl));
    assert(Json::parse(escaped.dump()).at("character") == "\"");
    assert(bornopata::mapKey(fcitx::Key(FcitxKey_ISO_Left_Tab))["modifiers"]["shift"] == true);
    assert(bornopata::mapKey(fcitx::Key(FcitxKey_Return))["virtual_key"] == 0x0d);
    char program[] = "borno-test"; char disable[] = "--disable=all";
    char *arguments[] = {program, disable, nullptr};
    fcitx::Instance instance(2, arguments);
    instance.initialize();
    fcitx::BornopataEngine engine(&instance.addonManager());
    auto owned = std::make_unique<Context>(instance.inputContextManager());
    auto &context = *owned;
    context.setCapabilityFlags(fcitx::CapabilityFlag::Preedit);
    context.focusIn();
    const bool consumed = engine.process(&context, fcitx::Key(FcitxKey_a));
    if (mode == "mismatch") {
        assert(!consumed && context.inputPanel().empty() && context.committed.empty());
        return 0;
    }
    assert(consumed);
    // The Bangla word is the client preedit; the Roman letters are in the panel.
    assert(context.inputPanel().clientPreedit().toString() == "আমি");
    assert(context.inputPanel().preedit().toString() == "ami");
    auto candidates = context.inputPanel().candidateList();
    assert(candidates && candidates->size() == 3);
    assert(candidates->candidate(0).text().toString() == "আমি");
    assert(candidates->candidate(2).comment().toString() == "★");
    assert(candidates->candidate(0).comment().toString().empty());
    if (mode == "failure") {
        mock.join();
        auto timer = instance.eventLoop().addTimeEvent(CLOCK_MONOTONIC, fcitx::now(CLOCK_MONOTONIC) + 50000, 0,
            [&](fcitx::EventSourceTime *, uint64_t) { instance.eventLoop().exit(); return false; });
        instance.eventLoop().exec();
        assert(!context.inputPanel().candidateList() && context.inputPanel().clientPreedit().empty());
        candidates->candidate(1).select(&context);
        assert(context.committed.empty());
    } else {
        candidates->candidate(1).select(&context);
        assert(context.committed == "অমি");
        // The frame is gone after the commit; a second click on the old list does nothing.
        candidates->candidate(1).select(&context);
        assert(context.committed == "অমি");
    }
    owned.reset();
    if (mode != "failure") {
        // Destroying the context queues a Close; one pass of the event loop sends it and ends the mock's loop.
        auto timer = instance.eventLoop().addTimeEvent(CLOCK_MONOTONIC, fcitx::now(CLOCK_MONOTONIC) + 30000, 0,
            [&](fcitx::EventSourceTime *, uint64_t) { instance.eventLoop().exit(); return false; });
        instance.eventLoop().exec();
    }
    mock.join();
}
