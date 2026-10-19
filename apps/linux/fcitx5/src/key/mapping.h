//! Mapping between Fcitx key symbols and the protocol key codes.
#pragma once
#include <fcitx-utils/key.h>
#include <nlohmann/json.hpp>
namespace bornopata {
nlohmann::json mapKey(const fcitx::Key &key);
}
