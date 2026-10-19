//! The real presenter, or `nullptr` when the plugin was built without Wayland support or the Wayland addon is not running.
#pragma once
#include "popup/presenter.h"
#include <memory>
#include <fcitx/addonmanager.h>
namespace bornopata {
std::unique_ptr<Presenter> makeWaylandPresenter(fcitx::AddonManager *manager);
}
