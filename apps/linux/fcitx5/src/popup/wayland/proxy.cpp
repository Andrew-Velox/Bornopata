//! Memory is read with process_vm_readv, so a stray word of the wrapper that is not a pointer, or points nowhere, is skipped instead of crashing the input method.
#include "proxy.h"
#include <cstdint>
#include <cstring>
#include <vector>
#include <sys/uio.h>
#include <unistd.h>
namespace bornopata {
namespace {
/// Copies `size` bytes from this process's own memory at `from`; `false` when any of it is not readable.
bool copy(const void *from, void *to, size_t size) {
    iovec local{to, size}, remote{const_cast<void *>(from), size};
    return process_vm_readv(getpid(), &local, 1, &remote, 1, 0) == static_cast<ssize_t>(size);
}

/// The part of a `wl_proxy` (interface, display, queue, flags, user data) that is read; every proxy is larger.
constexpr size_t PROXY_BYTES = 64;
}

wl_proxy *findProxy(const void *wrapper, const char *interface, size_t bytes) {
    std::vector<unsigned char> object(bytes);
    // The object may be shorter than `bytes` and end at an unmapped page: take the largest prefix that can be read.
    while (bytes >= sizeof(void *) && !copy(wrapper, object.data(), bytes)) bytes /= 2;
    for (size_t at = 0; at + sizeof(void *) <= bytes; at += sizeof(void *)) {
        void *candidate;
        std::memcpy(&candidate, object.data() + at, sizeof(candidate));
        if (!candidate || (reinterpret_cast<uintptr_t>(candidate) & (sizeof(void *) - 1))) continue;
        unsigned char probe[PROXY_BYTES];
        if (!copy(candidate, probe, sizeof(probe))) continue;
        auto *proxy = static_cast<wl_proxy *>(candidate);
        if (wl_proxy_get_user_data(proxy) != wrapper) continue;
        const char *name = wl_proxy_get_class(proxy);
        // The class name must be the interface and then its terminating zero.
        const size_t length = std::strlen(interface) + 1;
        std::vector<char> text(length);
        if (name && copy(name, text.data(), length) && std::memcmp(text.data(), interface, length) == 0) return proxy;
    }
    return nullptr;
}
}
