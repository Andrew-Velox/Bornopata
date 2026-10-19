//! Default candidate panel. When the connection fails the preedit is cleared and input passes through; nothing is replayed.
#include "bornopata.h"
#include "candidate/word.h"
#include "popup/wayland/make.h"
#include "key/mapping.h"
#include <fcitx/addonmanager.h>
#include <fcitx/candidatelist.h>
#include <fcitx/inputcontext.h>
#include <fcitx/inputcontextmanager.h>
#include <fcitx/inputpanel.h>
#include <fcitx/userinterfacemanager.h>
#include <iomanip>
#include <algorithm>
#include <sstream>
#include <stdexcept>
#include <sys/socket.h>
#include <cerrno>

namespace fcitx {
namespace {
/// The wire protocol version; the Server refuses any other.
constexpr int PROTOCOL = 4;

/// Candidates the panel shows at once; the Server never sends more.
constexpr size_t PANEL_CANDIDATES = 9;

std::string contextIdentity(const InputContext *context) {
    std::ostringstream out;
    out << std::hex << std::setfill('0');
    for (auto byte : context->uuid()) out << std::setw(2) << static_cast<unsigned>(byte);
    return out.str();
}

/// A string member of an object, or empty when it is missing or not a string: a gloss that does not fit must never
/// bring the connection down.
std::string text(const nlohmann::json &object, const char *name) {
    auto found = object.find(name);
    return found != object.end() && found->is_string() ? found->get<std::string>() : "";
}

/// Annotation shown beside a candidate: a star for what the user chose before, then the gloss with its part of speech
/// ("n. rice") when the Server sent one.
std::string annotation(const nlohmann::json &item) {
    std::string note = item.value("source", "") == "Learned" ? "★" : "";
    auto gloss = item.find("gloss");
    if (gloss == item.end() || !gloss->is_object()) return note;
    const auto meaning = text(*gloss, "text");
    if (meaning.empty()) return note;
    const auto pos = text(*gloss, "pos");
    if (!note.empty()) note += " ";
    return note + (pos.empty() ? "" : pos + " ") + meaning;
}
}

BornopataEngine::BornopataEngine(AddonManager *manager)
    : BornopataEngine(manager, bornopata::makeWaylandPresenter(manager)) {}

BornopataEngine::BornopataEngine(AddonManager *manager, std::unique_ptr<bornopata::Presenter> presenter)
    : instance_(manager->instance()), shared_(std::make_shared<bornopata::SharedConnection>()),
      sessions_([this](InputContext &context) {
          const auto id = shared_->nextSession++;
          shared_->contexts.emplace(id, context.watch());
          return new bornopata::Session(shared_, id);
      }) {
    if (presenter)
        popup_ = std::make_unique<bornopata::PopupBridge>(instance_->eventLoop(), std::move(presenter),
            [this](const std::string &identity) -> InputContext * {
                for (const auto &[id, watched] : shared_->contexts) {
                    (void)id;
                    auto *context = watched.get();
                    if (!context || contextIdentity(context) != identity) continue;
                    auto *session = context->propertyFor(&sessions_);
                    return context->hasFocus() && session->opened && session->showing ? context : nullptr;
                }
                return nullptr;
            });
    shared_->deferred = instance_->eventLoop().addDeferEvent([this](EventSource *event) {
        event->setEnabled(false);
        if (shared_->connection.connected() && !shared_->flush()) disconnectAll();
        return true;
    });
    shared_->deferred->setEnabled(false);
    instance_->inputContextManager().registerProperty("borno-session", &sessions_);
    capabilityWatcher_ = instance_->watchEvent(EventType::InputContextCapabilityChanged, EventWatcherPhase::PreInputMethod, [this](Event &event) {
        auto *context = static_cast<InputContextEvent &>(event).inputContext();
        ++context->propertyFor(&sessions_)->lifecycle;
        if (context->propertyFor(&sessions_)->opened) syncPrivacy(context);
    });
    cursorWatcher_ = instance_->watchEvent(EventType::InputContextCursorRectChanged, EventWatcherPhase::PreInputMethod, [this](Event &event) {
        auto *context = static_cast<InputContextEvent &>(event).inputContext();
        if (context->propertyFor(&sessions_)->opened && context->hasFocus()) syncCursor(context);
    });
    focusWatcher_ = instance_->watchEvent(EventType::InputContextFocusOut, EventWatcherPhase::PreInputMethod, [this](Event &event) {
        auto *context = static_cast<InputContextEvent &>(event).inputContext();
        auto *session = context->propertyFor(&sessions_);
        ++session->lifecycle;
        session->focused = false;
        session->showing = false;
        if (popup_) popup_->hide(context);
        if (session->opened) exchange(context, {{"Focus", {{"focused", false}}}}, false);
    });
}

BornopataEngine::~BornopataEngine() {
    *alive_ = false;
    popup_.reset();
    capabilityWatcher_.reset(); focusWatcher_.reset(); cursorWatcher_.reset();
    shared_->watcher.reset(); shared_->deferred.reset(); shared_->connection.close();
    const auto contexts = shared_->contexts;
    shared_->contexts.clear();
    for (const auto &[id, watched] : contexts) {
        (void)id;
        if (auto *context = watched.get()) {
            context->propertyFor(&sessions_)->opened = false;
            context->inputPanel().reset();
        }
    }
    sessions_.unregister();
}

void BornopataEngine::clear(InputContext *context) {
    const auto watched = context->watch();
    auto *session = context->propertyFor(&sessions_);
    const auto revision = ++session->revision;
    session->displayIdentity = nullptr;
    session->clientPreedit = false;
    session->showing = false;
    if (popup_) popup_->hide(context);
    context->inputPanel().reset();
    context->updatePreedit();
    if (watched.get() && session->revision == revision)
        context->updateUserInterface(UserInterfaceComponent::InputPanel);
}

void BornopataEngine::disconnectAll() {
    if (shared_->clearing) return;
    shared_->clearing = true;
    if (popup_) popup_->reset();
    shared_->watcher.reset();
    shared_->connection.close();
    shared_->retired.clear();
    const auto contexts = shared_->contexts;
    // Invalidate every session first, then call the reentrant UI; reconnecting is forbidden while clearing.
    for (const auto &[id, watched] : contexts) {
        (void)id;
        if (auto *context = watched.get()) {
            auto *session = context->propertyFor(&sessions_);
            session->opened = false;
            session->focused = false;
            session->capabilities = nullptr;
            ++session->revision;
        }
    }
    for (const auto &[id, watched] : contexts)
        if (auto *context = watched.get(); context && shared_->contexts.contains(id)) clear(context);
    shared_->clearing = false;
}

void BornopataEngine::disconnect(InputContext *context) {
    auto *session = context->propertyFor(&sessions_);
    if (session->opened) {
        shared_->retire(session->id);
        session->id = shared_->nextSession++;
        shared_->contexts.emplace(session->id, context->watch());
    }
    session->opened = false;
    session->focused = false;
    session->capabilities = nullptr;
    clear(context);
}

bool BornopataEngine::connect(InputContext *context) {
    auto *session = context->propertyFor(&sessions_);
    const auto watched = context->watch();
    if (shared_->clearing) return false;
    if (session->opened) {
        if (session->focused != context->hasFocus()) {
            session->focused = context->hasFocus();
            exchange(context, {{"Focus", {{"focused", session->focused}}}}, false);
        }
        return watched.get() && session->opened;
    }
    try {
        if (!shared_->connection.connected()) {
            if (!shared_->connection.open()) return false;
            ++shared_->generation;
            shared_->retired.clear();
            shared_->watcher = instance_->eventLoop().addIOEvent(shared_->connection.fd(), IOEventFlag::In,
                [this](EventSourceIO *, int fd, IOEventFlags) {
                    char byte;
                    auto count = recv(fd, &byte, 1, MSG_PEEK | MSG_DONTWAIT);
                    if (count >= 0 || (errno != EAGAIN && errno != EWOULDBLOCK && errno != EINTR)) disconnectAll();
                    return true;
                });
        }
        if (!shared_->flush()) throw std::runtime_error("close exchange");
        session->generation = shared_->generation;
        nlohmann::json response;
        // A context that can show the popup names the channel the Server draws its frames in.
        const auto popup = popup_ ? popup_->token(context, shared_->generation) : std::nullopt;
        if (!shared_->connection.send({{"Open", {{"session", session->id}, {"app", context->program()}, {"protocol", PROTOCOL},
                {"generation", session->generation}, {"context", contextIdentity(context)},
                {"popup", popup ? nlohmann::json(*popup) : nlohmann::json(nullptr)}}}}, &response)
            || response.at("Opened").at("session") != session->id
            || response.at("Opened").at("protocol") != PROTOCOL)
            throw std::runtime_error("protocol mismatch");
        session->mode = response.at("Opened").at("mode").get<std::string>();
        session->opened = true;
        if (!syncPrivacy(context)) return false;
        session->focused = context->hasFocus();
        exchange(context, {{"Focus", {{"focused", session->focused}}}}, false);
        return watched.get() && session->opened;
    } catch (const std::exception &) { disconnectAll(); return false; }
}

bool BornopataEngine::syncPrivacy(InputContext *context) {
    auto *session = context->propertyFor(&sessions_);
    const auto caps = context->capabilityFlags();
    nlohmann::json facts = {{"sensitive", caps.test(CapabilityFlag::Sensitive)},
        {"password", caps.test(CapabilityFlag::Password)}, {"disabled", caps.test(CapabilityFlag::Disable)}};
    if (session->capabilities != facts) {
        const auto watched = context->watch();
        const auto generation = session->generation;
        // The Server confirms the capabilities first, then the UI is cleared, which may reenter.
        // The Reset, Focus and key events sent from clear() already see the new privacy state; a nested update
        // overwrites ours, so the outer call must not write the cache again after it returns.
        exchange(context, {{"Capabilities", facts}}, false);
        if (!watched.get() || !session->opened || session->generation != generation) return false;
        session->capabilities = facts;
        session->clientPreedit = false;
        clear(context);
        if (!watched.get() || !session->opened) return false;
    }
    // clear() can reenter; decide whether to close this session from the capabilities as they are now.
    const auto currentCaps = context->capabilityFlags();
    if (currentCaps.test(CapabilityFlag::Password) || currentCaps.test(CapabilityFlag::Disable)) {
        disconnect(context);
        return false;
    }
    return session->opened;
}

void BornopataEngine::syncCursor(InputContext *context) {
    auto *session = context->propertyFor(&sessions_);
    const auto &rect = context->cursorRect();
    // X11 clients (also through XWayland) report global coordinates; everything else is relative to the surface.
    const bool surfaceRelative = context->display().rfind("x11", 0) != 0;
    nlohmann::json cursor = {{"x", rect.left()}, {"y", rect.top()}, {"width", rect.width()}, {"height", rect.height()},
        {"surface_relative", surfaceRelative}};
    if (session->cursor == cursor) return;
    session->cursor = cursor;
    exchange(context, {{"Cursor", cursor}}, false);
}

bool BornopataEngine::exchange(InputContext *context, const nlohmann::json &event, bool display) {
    auto *session = context->propertyFor(&sessions_);
    const auto watched = context->watch();
    const auto generation = session->generation;
    const auto lifecycle = session->lifecycle;
    const auto capabilities = context->capabilityFlags();
    const bool focused = context->hasFocus();
    bool consumed = false;
    try {
        nlohmann::json response;
        if (!session->opened || !shared_->connection.send({{"Event", {{"session", session->id}, {"event", event}}}}, &response))
            throw std::runtime_error("event exchange");
        if (response.contains("Ignored") && response.at("Ignored").at("session") == session->id) return true;
        const auto &result = response.at("Result");
        const auto &identity = result.at("identity");
        const auto outcome = result.at("outcome").get<std::string>();
        const auto &commit = result.at("commit");
        if (result.at("session") != session->id || (outcome != "Consumed" && outcome != "Passthrough")
            || (!commit.is_null() && !commit.is_string()) || identity.at("generation") != session->generation
            || identity.at("context") != contextIdentity(context) || !identity.at("revision").is_number_unsigned()
            || (!session->displayIdentity.is_null() && identity.at("revision").get<uint64_t>() <= session->displayIdentity.at("revision").get<uint64_t>()))
            throw std::runtime_error("response identity");
        consumed = outcome == "Consumed";
        session->displayIdentity = identity;
        // Commit before clearing the preedit, or native Wayland Chromium types the text twice (#213).
        // The commit callback can reenter too; only a context that still owns this frame may go on drawing.
        const auto revision = session->revision;
        auto current = [&] {
            return watched.get() && session->opened && session->generation == generation
                && session->lifecycle == lifecycle && session->revision == revision
                && session->displayIdentity == identity && context->hasFocus() == focused
                && context->capabilityFlags() == capabilities;
        };
        if (commit.is_string() && current()) context->commitString(commit.get<std::string>());
        const auto &frame = result.at("frame");
        const auto mode = frame.at("mode").get<std::string>();
        const bool switched = !session->mode.empty() && session->mode != mode;
        session->mode = mode;
        if (display && current()) render(context, frame);
        // A short popup tells the user which mode the toggle selected.
        if (switched && current()) instance_->showCustomInputMethodInformation(context, mode == "English" ? "English" : "বাংলা");
        return outcome == "Consumed";
    } catch (const std::exception &) { disconnectAll(); return consumed; }
}

void BornopataEngine::reset(const InputMethodEntry &, InputContextEvent &event) {
    auto *context = event.inputContext();
    const auto watched = context->watch();
    ++context->propertyFor(&sessions_)->lifecycle;
    if (context->propertyFor(&sessions_)->opened) exchange(context, "Reset", false);
    if (!watched.get()) return;
    context->propertyFor(&sessions_)->clientPreedit = false;
    clear(context);
}

void BornopataEngine::deactivate(const InputMethodEntry &, InputContextEvent &event) {
    auto *context = event.inputContext();
    auto *session = context->propertyFor(&sessions_);
    const auto watched = context->watch();
    ++session->lifecycle;
    auto *switched = dynamic_cast<InputContextSwitchInputMethodEvent *>(&event);
    // Forward the reason the framework gave; the capabilities may still be the old ones at this point.
    if (session->opened) {
        bool clientPreedit = session->clientPreedit;
        syncPrivacy(context);
        if (!watched.get()) return;
        if (session->opened) exchange(context, {{"Deactivate", {
            {"focus_out", event.type() == EventType::InputContextFocusOut}, {"client_preedit", clientPreedit},
            {"capability_changed", switched && switched->reason() == InputMethodSwitchedReason::CapabilityChanged}}}}, false);
    }
    if (!watched.get()) return;
    session->focused = false;
    session->clientPreedit = false;
    clear(context);
}

void BornopataEngine::keyEvent(const InputMethodEntry &, KeyEvent &event) {
    if (event.inputContext() && process(event.inputContext(), event.rawKey(), event.isRelease())) event.filterAndAccept();
}

bool BornopataEngine::process(InputContext *context, const Key &key, bool release) {
    const auto watched = context->watch();
    const auto caps = context->capabilityFlags();
    if (caps.test(CapabilityFlag::Password) || caps.test(CapabilityFlag::Disable)) {
        if (context->propertyFor(&sessions_)->opened) syncPrivacy(context);
        return false;
    }
    if (!connect(context) || !watched.get() || !syncPrivacy(context) || !watched.get()) return false;
    syncCursor(context);
    if (!watched.get()) return false;
    return exchange(context, {{"Key", {{"event", bornopata::mapKey(key)}, {"release", release}}}});
}

void BornopataEngine::render(InputContext *context, const nlohmann::json &frame) {
    auto *session = context->propertyFor(&sessions_);
    const auto identity = session->displayIdentity;
    const auto revision = ++session->revision;
    const auto roman = frame.at("roman").get<std::string>();
    const auto bangla = frame.at("preedit").get<std::string>();
    const bool suggesting = frame.at("suggesting").get<bool>();
    const auto &items = frame.at("candidates");
    if (!items.is_array() || items.size() > PANEL_CANDIDATES) throw std::runtime_error("candidate count");

    // The word as Bangla goes into the application as underlined preedit; the Roman letters stay in the panel.
    // Without client preedit support the panel shows both.
    const bool inlinePreedit = context->capabilityFlags().test(CapabilityFlag::Preedit);
    Text clientPreedit;
    if (!bangla.empty()) clientPreedit.append(bangla, TextFormatFlag::Underline);
    clientPreedit.setCursor(static_cast<int>(clientPreedit.toString().size()));
    Text panelPreedit;
    if (!roman.empty()) panelPreedit.append(inlinePreedit || bangla.empty() ? roman : roman + "  " + bangla);
    panelPreedit.setCursor(static_cast<int>(panelPreedit.toString().size()));

    auto watched = context->watch();
    const auto lifecycle = session->lifecycle;
    auto current = [&] {
        return watched.get() && session->opened && session->lifecycle == lifecycle
            && session->revision == revision && session->displayIdentity == identity;
    };
    // When the Server shows the frame in its own window the Fcitx5 panel stays empty; only the inline preedit remains.
    const bool native = frame.value("native", false);
    session->showing = native && (!items.empty() || !roman.empty());
    auto list = std::make_unique<CommonCandidateList>();
    list->setPageSize(PANEL_CANDIDATES);
    if (suggesting) list->setLabels({"Tab", "", "", "", "", "", "", "", ""});
    else list->setLabels({"1", "2", "3", "4", "5", "6", "7", "8", "9"});
    size_t index = 0;
    for (const auto &item : items) {
        list->append(std::make_unique<bornopata::Word>(item.at("text").get<std::string>(), annotation(item),
            [this, alive = alive_, watched, index, revision, identity](InputContext *ic) {
                if (*alive && ic && ic == watched.get() && ic->hasFocus() && ic->propertyFor(&sessions_)->revision == revision)
                    exchange(ic, {{"Candidate", {{"identity", identity}, {"index", index}}}});
            }));
        ++index;
    }
    const auto highlight = frame.at("highlight").get<size_t>();
    if (highlight < items.size()) list->setGlobalCursorIndex(static_cast<int>(highlight));

    auto &panel = context->inputPanel();
    panel.reset();
    if (inlinePreedit) panel.setClientPreedit(clientPreedit);
    if (!native) panel.setPreedit(panelPreedit);
    session->clientPreedit = inlinePreedit && !clientPreedit.empty();
    if (!items.empty() && !native) panel.setCandidateList(std::move(list));
    context->updatePreedit();
    if (!current()) return;
    context->updateUserInterface(UserInterfaceComponent::InputPanel);
    if (!current()) return;
    instance_->userInterfaceManager().flush();
}
}
