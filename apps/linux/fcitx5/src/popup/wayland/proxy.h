//! Finding the raw Wayland proxy inside an object of Fcitx5's C++ wrapper classes, which are not installed.
#pragma once
#include <cstddef>
#include <wayland-client.h>
namespace bornopata {
/// The proxy of `interface` that `wrapper` owns, or `nullptr`. Fcitx5's wrapper stores itself as the user data of its proxy, so a readable pointer inside the object whose user data is the object and whose class is `interface` is the proxy; no layout is assumed. `bytes` bounds the search.
wl_proxy *findProxy(const void *wrapper, const char *interface, size_t bytes = 1024);
}
