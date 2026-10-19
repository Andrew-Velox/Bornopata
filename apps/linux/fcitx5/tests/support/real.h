//! An isolated real Rust Server: config, learned data, logs and the socket all live in a temporary directory.
#pragma once
#include <cassert>
#include <chrono>
#include <filesystem>
#include <fstream>
#include <thread>
#include <signal.h>
#include <sys/wait.h>
#include <unistd.h>
class RealServer {
public:
    std::filesystem::path directory;

    pid_t pid = -1;

    /// `executable` is the built `borno-linux-server`; `config` is appended to the config file.
    explicit RealServer(const char *executable, const std::string &config = "") {
        char pattern[] = "/tmp/borno-real-XXXXXX";
        auto *created = mkdtemp(pattern); assert(created);
        directory = created;
        setenv("BORNOPATA_SOCKET", (directory / "server.sock").c_str(), 1);
        setenv("XDG_CONFIG_HOME", directory.c_str(), 1);
        setenv("XDG_DATA_HOME", directory.c_str(), 1);
        setenv("XDG_STATE_HOME", directory.c_str(), 1);
        unsetenv("DBUS_SESSION_BUS_ADDRESS");
        std::filesystem::create_directories(directory / "bornopata");
        std::ofstream(directory / "bornopata/config.toml") << config;
        pid = fork(); assert(pid >= 0);
        if (pid == 0) { execl(executable, executable, nullptr); _exit(127); }
        for (int i = 0; i < 500 && !std::filesystem::exists(directory / "server.sock"); ++i)
            std::this_thread::sleep_for(std::chrono::milliseconds(10));
        assert(std::filesystem::exists(directory / "server.sock"));
    }
    /// Stops the Server the way a session end does (SIGTERM) and returns whether it exited cleanly.
    bool stop() {
        if (pid <= 0) return true;
        kill(pid, SIGTERM);
        int status = 0;
        waitpid(pid, &status, 0);
        pid = -1;
        return WIFEXITED(status) && WEXITSTATUS(status) == 0;
    }
    std::string read(const std::string &relative) const {
        std::ifstream input(directory / relative);
        return std::string(std::istreambuf_iterator<char>(input), std::istreambuf_iterator<char>());
    }
    ~RealServer() { stop(); std::filesystem::remove_all(directory); }
};
