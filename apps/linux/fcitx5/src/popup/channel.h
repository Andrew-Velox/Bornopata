//! The popup channel: a second connection to the Server that it pushes bitmaps through; read without blocking from the event loop.
#pragma once
#include <cstdint>
#include <deque>
#include <string>
#include <vector>
#include <nlohmann/json.hpp>
namespace bornopata {
class Channel {
public:
    /// A message from the Server; `fd` is the descriptor that came with a `Pool` message, owned by the receiver (-1 for others).
    struct Message {
        nlohmann::json body;

        int fd = -1;
    };

    Channel() = default;
    Channel(const Channel &) = delete;
    Channel &operator=(const Channel &) = delete;
    ~Channel();

    /// Connects and attaches with `token`; `false` when the Server refuses or the protocol differs.
    bool open(uint64_t token);
    void close();
    bool connected() const { return fd_ >= 0; }
    int fd() const { return fd_; }

    /// Sends one message without waiting; `false` closes the channel.
    bool send(const nlohmann::json &message);

    /// Appends every complete message that has arrived to `out`. `false` when the Server hung up or sent something that is not a message; the channel is closed then, and `out` still has what arrived before.
    bool read(std::vector<Message> &out);
private:
    int fd_ = -1;

    std::string pending_;

    /// Descriptors received and not yet handed to a `Pool` message.
    std::deque<int> fds_;
};
}
