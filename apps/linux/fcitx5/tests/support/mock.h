//! A scriptable Server stand-in: it returns canned frames and records what the plugin sent.
#pragma once
#include "wire.h"
#include <cstring>
#include <filesystem>
#include <thread>
#include <sys/un.h>
#include <unistd.h>
namespace bornopata::test {
struct Mock {
    std::string directory;

    int listener;

    std::thread thread;

    std::vector<Json> messages;

    /// Modes: "both" is the normal Server, "mismatch" answers with another protocol version,
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
        thread = std::thread([this, mode] { serve(mode); });
    }
    ~Mock() {
        join();
        close(listener);
        std::filesystem::remove_all(directory);
    }
    void join() { if (thread.joinable()) thread.join(); }
    void serve(const std::string &mode) {
        int fd = accept(listener, nullptr, nullptr);
        auto opened = readMessage(fd).at("Open");
        const auto session = opened.at("session");
        assert(opened.at("protocol") == 1);
        writeMessage(fd, {{"Opened", {{"session", session}, {"protocol", mode == "mismatch" ? 2 : 1}, {"mode", "Bangla"}}}});
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
                outcome = "Consumed";
            }
            if (event.contains("Candidate")) {
                assert(event.at("Candidate").at("index") == 1);
                assert(event.at("Candidate").at("identity") == identity);
                commit = frame.at("candidates").at(1).at("text");
                frame = emptyFrame();
                outcome = "Consumed";
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
