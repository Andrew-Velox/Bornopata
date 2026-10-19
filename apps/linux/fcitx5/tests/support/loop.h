//! Runs the Fcitx5 event loop for a moment, so channel and socket events are delivered. `EventLoop::exec` can run only once (the loop refuses to start again after an exit), so this steps the loop itself.
#pragma once
#include <algorithm>
#include <cassert>
#include <cstring>
#include <fcitx/instance.h>
#include <systemd/sd-event.h>
inline void pump(fcitx::Instance &instance, uint64_t micros = 60000) {
    assert(std::strcmp(instance.eventLoop().implementation(), "sd-event") == 0);
    auto *loop = static_cast<sd_event *>(instance.eventLoop().nativeHandle());
    const auto end = fcitx::now(CLOCK_MONOTONIC) + micros;
    for (auto now = fcitx::now(CLOCK_MONOTONIC); now < end; now = fcitx::now(CLOCK_MONOTONIC))
        sd_event_run(loop, std::min<uint64_t>(end - now, 10000));
}
