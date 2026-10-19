//! An input context without an application, for default panel tests.
#pragma once
#include <fcitx/inputcontext.h>
#include <functional>
#include <utility>
class Context final : public fcitx::InputContext {
public:
    explicit Context(fcitx::InputContextManager &manager) : InputContext(manager, "panel-test") { created(); }
    ~Context() override { destroy(); }
    const char *frontend() const override { return "test"; }
    std::string committed;

    std::function<void()> onPreedit;

    std::function<void()> onCommit;
protected:
    void commitStringImpl(const std::string &text) override {
        committed += text;
        auto callback = std::move(onCommit);
        if (callback) callback();
    }
    void deleteSurroundingTextImpl(int, unsigned int) override {}
    void forwardKeyImpl(const fcitx::ForwardKeyEvent &) override {}
    void updatePreeditImpl() override {
        // The callback fires once. Not std::move: libc++ does not guarantee a moved-from std::function is empty.
        auto callback = std::exchange(onPreedit, nullptr);
        if (callback) callback();
    }
};
