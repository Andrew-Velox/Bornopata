//! Finding the raw proxy inside an object that is laid out unknown: right place, wrong owner, wrong interface, garbage words and the end of readable memory.
#include "popup/wayland/proxy.h"
#include <cassert>
#include <cstring>
#include <sys/mman.h>
#include <sys/socket.h>
#include <unistd.h>

namespace {
/// Stands for Fcitx5's wrapper: some words, then the proxy somewhere among them.
struct Wrapper {
    void *words[40];
};
}

int main() {
    // A display on a socket nobody serves is enough to make proxies; requests are only queued.
    int fds[2];
    assert(socketpair(AF_UNIX, SOCK_STREAM, 0, fds) == 0);
    wl_display *display = wl_display_connect_to_fd(fds[0]);
    assert(display);
    auto *registry = reinterpret_cast<wl_proxy *>(wl_display_get_registry(display));
    auto *other = reinterpret_cast<wl_proxy *>(wl_display_get_registry(display));

    Wrapper wrapper{};
    for (auto &word : wrapper.words) word = reinterpret_cast<void *>(uintptr_t(0xdead0001));
    wrapper.words[3] = reinterpret_cast<void *>(0x1000);
    wrapper.words[7] = other;
    wrapper.words[17] = registry;
    wl_proxy_set_user_data(registry, &wrapper);
    wl_proxy_set_user_data(other, nullptr);
    // The proxy is the pointer whose user data is the wrapper, wherever it is; a pointer to another proxy and garbage are skipped.
    assert(bornopata::findProxy(&wrapper, "wl_registry") == registry);
    // Another interface is not it, and neither is a proxy owned by something else.
    assert(!bornopata::findProxy(&wrapper, "wl_compositor"));
    wl_proxy_set_user_data(registry, nullptr);
    assert(!bornopata::findProxy(&wrapper, "wl_registry"));
    // A word that is a pointer to a proxy whose user data is the wrapper but that is not at the search range is not found.
    wl_proxy_set_user_data(registry, &wrapper);
    assert(!bornopata::findProxy(&wrapper, "wl_registry", 8 * 10));
    assert(bornopata::findProxy(&wrapper, "wl_registry", 8 * 18) == registry);

    // The object ends where readable memory ends: the search must not touch what is behind it.
    const size_t page = sysconf(_SC_PAGESIZE);
    auto *memory = static_cast<char *>(mmap(nullptr, 2 * page, PROT_READ | PROT_WRITE, MAP_PRIVATE | MAP_ANONYMOUS, -1, 0));
    assert(memory != MAP_FAILED && mprotect(memory + page, page, PROT_NONE) == 0);
    struct Small { void *words[4]; };
    auto *small = reinterpret_cast<Small *>(memory + page - sizeof(Small));
    small->words[0] = reinterpret_cast<void *>(0x42);
    small->words[1] = registry;
    small->words[2] = small->words[3] = nullptr;
    wl_proxy_set_user_data(registry, small);
    assert(bornopata::findProxy(small, "wl_registry") == registry);
    // Nothing found and nothing crashed with a wrapper full of non-pointers.
    std::memset(memory, 0xff, page);
    assert(!bornopata::findProxy(memory, "wl_registry"));
    assert(!bornopata::findProxy(nullptr, "wl_registry"));
    wl_display_disconnect(display);
}
