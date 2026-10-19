//! Fcitx5 thin plugin: forwards framework events to the Server and draws the frame it returns on the default panel.
#pragma once
#include "session.h"
#include <fcitx/addonfactory.h>
#include <fcitx/inputcontext.h>
#include <fcitx/inputmethodengine.h>
#include <fcitx/inputcontextproperty.h>
#include <fcitx/instance.h>
namespace fcitx {
class BornopataEngine final : public InputMethodEngineV2 {
public:
    explicit BornopataEngine(AddonManager *manager);
    ~BornopataEngine() override;
    void keyEvent(const InputMethodEntry &, KeyEvent &) override;
    void reset(const InputMethodEntry &, InputContextEvent &) override;
    void deactivate(const InputMethodEntry &, InputContextEvent &) override;
    bool process(InputContext *context, const Key &key, bool release = false);
private:
    void clear(InputContext *context);
    void disconnect(InputContext *context);
    void disconnectAll();
    bool connect(InputContext *context);
    bool syncPrivacy(InputContext *context);
    /// Tells the Server where the text cursor is, if it moved since the last time.
    void syncCursor(InputContext *context);
    bool exchange(InputContext *context, const nlohmann::json &event, bool display = true);
    void render(InputContext *context, const nlohmann::json &frame);
    Instance *instance_;
    std::shared_ptr<bornopata::SharedConnection> shared_;
    std::shared_ptr<bool> alive_ = std::make_shared<bool>(true);
    FactoryFor<bornopata::Session> sessions_;
    std::unique_ptr<HandlerTableEntry<EventHandler>> capabilityWatcher_;
    std::unique_ptr<HandlerTableEntry<EventHandler>> focusWatcher_;
    std::unique_ptr<HandlerTableEntry<EventHandler>> cursorWatcher_;
};
}
