//! Frames are the same as on the main connection: a little-endian 4-byte length and JSON. A `Pool` message carries one descriptor as SCM_RIGHTS.
#include "channel.h"
#include "ipc/connection.h"
#include <array>
#include <cerrno>
#include <cstring>
#include <sys/socket.h>
#include <unistd.h>
namespace bornopata {
namespace {
/// Frames the Server may send are small; a longer one is corrupt.
constexpr size_t MAX_FRAME = 1024 * 1024;

/// The protocol version; the Server refuses any other.
constexpr int PROTOCOL = 4;
}
Channel::~Channel() { close(); }

void Channel::close() {
    if (fd_ >= 0) ::close(fd_);
    fd_ = -1;
    pending_.clear();
    for (int fd : fds_) ::close(fd);
    fds_.clear();
}

bool Channel::open(uint64_t token) {
    close();
    Connection connection;
    nlohmann::json response;
    if (!connection.open()
        || !connection.send({{"Attach", {{"protocol", PROTOCOL}, {"token", token}}}}, &response)
        || !response.contains("Attached") || response.at("Attached").value("protocol", 0) != PROTOCOL)
        return false;
    fd_ = connection.release();
    return true;
}

bool Channel::send(const nlohmann::json &message) {
    if (fd_ < 0) return false;
    const std::string body = message.dump();
    const uint32_t size = static_cast<uint32_t>(body.size());
    std::string framed(4, '\0');
    for (int i = 0; i < 4; ++i) framed[i] = static_cast<char>(size >> (8 * i));
    framed += body;
    // A message is a few dozen bytes; a full socket buffer means the Server is gone or stuck, and either way the channel is not worth keeping.
    if (::send(fd_, framed.data(), framed.size(), MSG_NOSIGNAL | MSG_DONTWAIT) != static_cast<ssize_t>(framed.size())) {
        close();
        return false;
    }
    return true;
}

bool Channel::read(std::vector<Message> &out) {
    if (fd_ < 0) return false;
    bool hangup = false;
    for (;;) {
        std::array<char, 8192> bytes;
        alignas(cmsghdr) std::array<char, CMSG_SPACE(sizeof(int) * 4)> control{};
        iovec vector{bytes.data(), bytes.size()};
        msghdr header{};
        header.msg_iov = &vector;
        header.msg_iovlen = 1;
        header.msg_control = control.data();
        header.msg_controllen = control.size();
        const ssize_t count = recvmsg(fd_, &header, MSG_DONTWAIT | MSG_CMSG_CLOEXEC);
        if (count < 0 && errno == EINTR) continue;
        if (count < 0 && (errno == EAGAIN || errno == EWOULDBLOCK)) break;
        if (count <= 0) { hangup = true; break; }
        for (auto *part = CMSG_FIRSTHDR(&header); part; part = CMSG_NXTHDR(&header, part)) {
            if (part->cmsg_level != SOL_SOCKET || part->cmsg_type != SCM_RIGHTS) continue;
            const size_t size = (part->cmsg_len - CMSG_LEN(0)) / sizeof(int);
            for (size_t i = 0; i < size; ++i) {
                int fd;
                std::memcpy(&fd, CMSG_DATA(part) + i * sizeof(int), sizeof(int));
                fds_.push_back(fd);
            }
        }
        pending_.append(bytes.data(), static_cast<size_t>(count));
    }
    while (pending_.size() >= 4) {
        const uint32_t size = uint32_t(uint8_t(pending_[0])) | uint32_t(uint8_t(pending_[1])) << 8
            | uint32_t(uint8_t(pending_[2])) << 16 | uint32_t(uint8_t(pending_[3])) << 24;
        if (size == 0 || size > MAX_FRAME) { close(); return false; }
        if (pending_.size() < 4 + size) break;
        auto body = nlohmann::json::parse(pending_.substr(4, size), nullptr, false);
        pending_.erase(0, 4 + size);
        if (body.is_discarded()) { close(); return false; }
        Message message{std::move(body), -1};
        if (message.body.is_object() && message.body.contains("Pool")) {
            if (fds_.empty()) { close(); return false; }
            message.fd = fds_.front();
            fds_.pop_front();
        }
        out.push_back(std::move(message));
    }
    // What the Server sent before it hung up is still delivered, in `out`.
    if (hangup) { close(); return false; }
    return true;
}
}
