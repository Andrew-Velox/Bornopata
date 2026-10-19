//! An input context keeps only its session identity and display facts; all input state lives in the Server.
#pragma once
#include "ipc/shared.h"
#include <fcitx/inputcontextproperty.h>
#include <fcitx-utils/event.h>
namespace bornopata {
struct Session final : fcitx::InputContextProperty {
    Session(std::shared_ptr<SharedConnection> shared, uint64_t number)
        : id(number), owner(shared) {}
    ~Session() override {
        if (auto shared = owner.lock()) {
            if (opened) shared->retire(id);
            else shared->contexts.erase(id);
        }
    }

    /// Increases for the life of the plugin and is never reused.
    uint64_t id;

    /// The destructor only withdraws the registration and queues the close; it must not block or touch an engine
    /// that is already destroyed.
    std::weak_ptr<SharedConnection> owner;

    bool opened = false;

    bool focused = false;

    /// Whether the last frame was given to the client as preedit; the framework commits that on focus out.
    bool clientPreedit = false;

    /// The last frame was drawn by the Server (its popup or window), not by the Fcitx5 panel.
    bool showing = false;

    uint64_t revision = 0;

    uint64_t generation = 0;

    /// Counts framework boundaries; after a reentrant call it stops an older reply from committing or redrawing.
    uint64_t lifecycle = 0;

    nlohmann::json displayIdentity;

    /// The capabilities the Server has confirmed; a send that has not succeeded does not count as synchronized.
    nlohmann::json capabilities;

    /// "Bangla" or "English", as last reported by the Server; empty before the first reply.
    std::string mode;

    /// The text cursor rectangle last sent to the Server, to send it again only when it changes.
    nlohmann::json cursor;
};
}
