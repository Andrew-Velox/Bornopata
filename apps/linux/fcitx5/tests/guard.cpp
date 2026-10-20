//! What Fcitx5's pointer handler finds in the popup surface's user data: zeros where it reads, so it ignores the surface.
#include "popup/wayland/guard.h"
#include <cassert>
#include <cstdint>
#include <cstring>
#include <sys/socket.h>
#include <wayland-client.h>

int main() {
    // A display on a socket nobody serves is enough to make a proxy; requests are only queued.
    int fds[2];
    assert(socketpair(AF_UNIX, SOCK_STREAM, 0, fds) == 0);
    wl_display *display = wl_display_connect_to_fd(fds[0]);
    assert(display);
    auto *proxy = reinterpret_cast<wl_proxy *>(wl_display_get_registry(display));

    bornopata::Guard guard;
    guard.owner = &guard;
    wl_proxy_set_user_data(proxy, &guard);
    // The handler reads a pointer-sized field `offset` bytes into what it thinks is its wrapper; wherever that is in
    // the first 256 bytes, it must be null.
    auto *object = static_cast<unsigned char *>(wl_proxy_get_user_data(proxy));
    assert(object);
    for (size_t offset = 0; offset + sizeof(void *) <= sizeof(guard.foreign); offset += sizeof(void *)) {
        void *field;
        std::memcpy(&field, object + offset, sizeof(field));
        assert(field == nullptr);
    }
    // The plugin's own listener still finds its owner.
    assert(static_cast<bornopata::Guard *>(wl_proxy_get_user_data(proxy))->owner == &guard);
    wl_display_disconnect(display);
}
