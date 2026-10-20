//! A scriptable Server stand-in: it returns canned frames and records what the plugin sent.
#pragma once
#include "wire.h"
#include <atomic>
#include <cstring>
#include <filesystem>
#include <mutex>
#include <thread>
#include <poll.h>
#include <sys/mman.h>
#include <sys/un.h>
#include <unistd.h>
namespace bornopata::test {
struct Mock {
    std::string directory;

    int listener;

    std::thread thread;

    std::vector<Json> messages;

    /// The `Open` messages of the session connection.
    std::vector<Json> opens;

    uint64_t buffers = 0;

    /// The popup channel of the "popup" mode: what the plugin sent on it (guarded by `lock`).
    std::vector<Json> channelMessages;

    std::mutex lock;

    /// The token of the `Attach` the plugin sent, 0 before it.
    std::atomic<uint64_t> attached{0};

    /// The channel's descriptor, -1 before the plugin attached.
    std::atomic<int> channel{-1};

    std::atomic<bool> done{false};

    std::vector<std::thread> handlers;

    /// Modes: "both" is the normal Server, "native" marks frames as drawn by the Server's own window, "mismatch" answers with another protocol version,
    /// "failure" hangs up after the first key reply.
    explicit Mock(const std::string &mode) {
        char path[] = "/tmp/borno-plugin-XXXXXX";
        assert(mkdtemp(path));
        directory = path;
        std::string socketPath = directory + "/server.sock";
        setenv("BORNOPATA_SOCKET", socketPath.c_str(), 1);
        setenv("XDG_CONFIG_HOME", path, 1);
        setenv("XDG_DATA_HOME", path, 1);
        unsetenv("DBUS_SESSION_BUS_ADDRESS");
        listener = socket(AF_UNIX, SOCK_STREAM, 0);
        sockaddr_un address{}; address.sun_family = AF_UNIX;
        std::strcpy(address.sun_path, socketPath.c_str());
        assert(bind(listener, reinterpret_cast<sockaddr *>(&address), sizeof(address)) == 0);
        assert(listen(listener, 2) == 0);
        thread = std::thread([this, mode] { accepting(mode); });
    }
    ~Mock() {
        join();
        close(listener);
        std::filesystem::remove_all(directory);
    }
    void join() { if (thread.joinable()) thread.join(); }

    /// Every connection gets its own thread: the popup channel attaches while the session connection waits for its reply.
    void accepting(const std::string &mode) {
        while (!done) {
            pollfd waiting{listener, POLLIN, 0};
            if (poll(&waiting, 1, 20) <= 0) continue;
            int fd = accept(listener, nullptr, nullptr);
            if (fd < 0) break;
            handlers.emplace_back([this, fd, mode] { handle(fd, mode); });
        }
        for (auto &handler : handlers) handler.join();
    }
    void handle(int fd, const std::string &mode) {
        auto first = readMessage(fd);
        if (first.contains("Attach")) { popupChannel(fd, first.at("Attach")); return; }
        serve(fd, mode, first.at("Open"));
        done = true;
    }
    /// The popup channel: answers the attach, then records what the plugin reports until it hangs up.
    void popupChannel(int fd, const Json &attach) {
        assert(attach.at("protocol") == 4);
        writeMessage(fd, {{"Attached", {{"protocol", 4}}}});
        attached = attach.at("token");
        channel = fd;
        char byte;
        while (recv(fd, &byte, 1, MSG_PEEK) > 0) {
            auto message = readMessage(fd);
            std::lock_guard guard(lock);
            channelMessages.push_back(message);
        }
        channel = -1;
        close(fd);
    }
    /// Pushes a pool (one memfd of `pixel` bytes, 8x4) and a bitmap in it to the plugin, as the Server's render thread does.
    void push(const std::string &context, uint64_t buffer, uint8_t pixel) {
        int fd = channel;
        if (fd < 0) return;
        int memory = memfd_create("mock-pool", MFD_CLOEXEC);
        assert(memory >= 0 && ftruncate(memory, 4096) == 0);
        std::vector<uint8_t> bytes(8 * 4 * 4, pixel);
        assert(write(memory, bytes.data(), bytes.size()) == static_cast<ssize_t>(bytes.size()));
        sendWithFd(fd, {{"Pool", {{"pool", 1}, {"size", 4096}}}}, memory);
        close(memory);
        writeMessage(fd, {{"Present", {{"context", context}, {"pool", 1}, {"buffer", buffer}, {"offset", 0}, {"width", 8}, {"height", 4},
            {"stride", 32}, {"scale_120", 120}, {"logical_width", 8}, {"logical_height", 4}}}});
    }
    void hideBitmap() { if (channel >= 0) writeMessage(channel, "Hide"); }
    void serve(int fd, const std::string &mode, const Json &opened) {
        opens.push_back(opened);
        const auto session = opened.at("session");
        assert(opened.at("protocol") == 4);
        writeMessage(fd, {{"Opened", {{"session", session}, {"protocol", mode == "mismatch" ? 5 : 4}, {"mode", "Bangla"}}}});
        if (mode == "mismatch") { close(fd); return; }
        Json identity = {{"generation", opened.at("generation")}, {"context", opened.at("context")}, {"revision", 0}};
        auto frame = emptyFrame();
        char byte;
        while (recv(fd, &byte, 1, MSG_PEEK) > 0) {
            auto message = readMessage(fd);
            messages.push_back(message);
            if (message.contains("Close")) break;
            const auto &event = message.at("Event").at("event");
            Json commit = nullptr;
            std::string outcome = "Passthrough";
            if (event.contains("Key") && !event.at("Key").at("release").get<bool>()
                && event.at("Key").at("event").at("character") == "a") {
                frame = amiFrame();
                // The Server draws the window itself in this mode and tells the plugin so.
                if (mode == "native" || mode == "popup") frame["native"] = true;
                if (mode == "popup") push(opened.at("context"), ++buffers, 0x7f);
                outcome = "Consumed";
            }
            // A key the Server passes on to the application and that leaves nothing to show (Backspace after a space).
            if (mode == "popup" && event.contains("Key") && !event.at("Key").at("release").get<bool>()
                && event.at("Key").at("event").at("character") == "x") frame = emptyFrame();
            if (mode == "popup" && event.contains("Focus") && !event.at("Focus").at("focused").get<bool>()) {
                // A bitmap that was already on its way when the focus left, then the hide the Server sends for it.
                push(opened.at("context"), ++buffers, 0x11);
                hideBitmap();
            }
            if (event.contains("Candidate")) {
                assert(event.at("Candidate").at("index") == 1);
                assert(event.at("Candidate").at("identity") == identity);
                commit = frame.at("candidates").at(1).at("text");
                frame = emptyFrame();
                outcome = "Consumed";
                if (mode == "popup") hideBitmap();
            }
            if (event.contains("Capabilities") || event == "Reset") frame = emptyFrame();
            if (event.contains("Deactivate")) {
                const auto &facts = event.at("Deactivate");
                // Like the real Server: the framework commits client preedit itself on focus out.
                if (!facts.at("capability_changed").get<bool>() && !(facts.at("focus_out").get<bool>() && facts.at("client_preedit").get<bool>())
                    && !frame.at("preedit").get<std::string>().empty()) commit = frame.at("preedit");
                frame = emptyFrame();
            }
            identity["revision"] = identity.at("revision").get<uint64_t>() + 1;
            writeMessage(fd, {{"Result", {{"session", session}, {"identity", identity}, {"frame", frame}, {"commit", commit}, {"outcome", outcome}}}});
            if (mode == "failure" && event.contains("Key")) break;
        }
        close(fd);
    }
};
}
