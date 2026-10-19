//! Bounded Unix socket transport: any error closes the connection and the next key starts a new handshake.
#pragma once
#include <nlohmann/json.hpp>
namespace bornopata {
class Connection {
public:
    Connection() = default;
    Connection(const Connection &) = delete;
    Connection &operator=(const Connection &) = delete;
    ~Connection();
    bool open();
    bool send(const nlohmann::json &message, nlohmann::json *response = nullptr);
    bool connected() const { return fd_ >= 0; }
    /// For the event loop to notice a hang-up; all traffic still goes through send.
    int fd() const { return fd_; }
    void close();
private:
    /// The connection; -1 when not connected.
    int fd_ = -1;
};
}
