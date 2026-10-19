//! A presenter that records what the plugin asks of it, and reads the pixels of the pools it is given.
#pragma once
#include "popup/presenter.h"
#include <cassert>
#include <cstdio>
#include <map>
#include <string>
#include <vector>
#include <unistd.h>
class FakePresenter final : public bornopata::Presenter {
public:
    struct Shown {
        std::string context;

        bornopata::Bitmap bitmap;

        /// The first byte of the bitmap in its pool.
        uint8_t first = 0;
    };

    ~FakePresenter() override { for (auto &[id, pool] : pools) { (void)id; close(pool.first); } }
    bool available(fcitx::InputContext *) override { return canShow; }
    void addPool(uint64_t pool, int fd, uint64_t size) override { pools[pool] = {fd, size}; }
    bool show(fcitx::InputContext *context, const bornopata::Bitmap &bitmap) override {
        auto pool = pools.find(bitmap.pool);
        assert(pool != pools.end() && uint64_t(bitmap.offset) + uint64_t(bitmap.stride) * bitmap.height <= pool->second.second);
        uint8_t byte = 0;
        assert(pread(pool->second.first, &byte, 1, bitmap.offset) == 1);
        shown.push_back({context->uuid().empty() ? "" : identity(context), bitmap, byte});
        visible = true;
        return accept;
    }
    void hide() override { ++hides; visible = false; }

    /// The identifier the plugin gives the Server for an input context: its uuid in hex.
    static std::string identity(fcitx::InputContext *context) {
        std::string text;
        char digits[3];
        for (auto byte : context->uuid()) { snprintf(digits, sizeof(digits), "%02x", unsigned(byte)); text += digits; }
        return text;
    }

    /// Whether `available` says yes, and whether `show` takes the bitmap.
    bool canShow = true;

    bool accept = true;

    bool visible = false;

    int hides = 0;

    std::vector<Shown> shown;

    /// Pool number to descriptor and size.
    std::map<uint64_t, std::pair<int, uint64_t>> pools;
};
