//! Test message reading and writing; assertions only check the protocol adaptation, input logic is covered by the Rust tests.
#pragma once
#include <nlohmann/json.hpp>
#include <array>
#include <cassert>
#include <cstring>
#include <sys/socket.h>
using Json = nlohmann::json;
inline Json readMessage(int fd) {
    std::array<uint8_t, 4> prefix{};
    assert(recv(fd, prefix.data(), 4, MSG_WAITALL) == 4);
    uint32_t size = uint32_t(prefix[0]) | uint32_t(prefix[1]) << 8 | uint32_t(prefix[2]) << 16 | uint32_t(prefix[3]) << 24;
    assert(size > 0 && size < 100000);
    std::string body(size, '\0');
    assert(recv(fd, body.data(), size, MSG_WAITALL) == size);
    return Json::parse(body);
}
inline void writeMessage(int fd, const Json &message) {
    std::string body = message.dump();
    uint32_t size = body.size();
    std::array<uint8_t, 4> prefix{uint8_t(size), uint8_t(size >> 8), uint8_t(size >> 16), uint8_t(size >> 24)};
    assert(send(fd, prefix.data(), 4, MSG_NOSIGNAL) == 4);
    assert(send(fd, body.data(), size, MSG_NOSIGNAL) == size);
}
/// Writes one message with a file descriptor attached to it as SCM_RIGHTS.
inline void sendWithFd(int socketFd, const Json &message, int fd) {
    std::string body = message.dump();
    uint32_t size = body.size();
    std::string framed = {char(size), char(size >> 8), char(size >> 16), char(size >> 24)};
    framed += body;
    iovec vector{framed.data(), framed.size()};
    alignas(cmsghdr) char control[CMSG_SPACE(sizeof(int))] = {};
    msghdr header{};
    header.msg_iov = &vector;
    header.msg_iovlen = 1;
    header.msg_control = control;
    header.msg_controllen = sizeof(control);
    auto *part = CMSG_FIRSTHDR(&header);
    part->cmsg_level = SOL_SOCKET;
    part->cmsg_type = SCM_RIGHTS;
    part->cmsg_len = CMSG_LEN(sizeof(int));
    std::memcpy(CMSG_DATA(part), &fd, sizeof(int));
    assert(sendmsg(socketFd, &header, MSG_NOSIGNAL) == static_cast<ssize_t>(framed.size()));
}
inline Json emptyFrame() {
    return Json::parse(R"({"roman":"","preedit":"","candidates":[],"highlight":0,"suggesting":false,"mode":"Bangla"})");
}
/// The frame the mock shows after the letters "ami": Bangla preedit and three candidates.
inline Json amiFrame() {
    return Json::parse(R"({"roman":"ami","preedit":"\u0986\u09ae\u09bf","candidates":[
        {"text":"\u0986\u09ae\u09bf","source":"Direct"},{"text":"\u0985\u09ae\u09bf","source":"Dictionary"},
        {"text":"\u0986\u09ae\u09c0","source":"Learned"}],"highlight":0,"suggesting":false,"mode":"Bangla"})");
}
