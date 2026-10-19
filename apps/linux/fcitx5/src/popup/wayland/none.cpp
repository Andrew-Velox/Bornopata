//! Built when the Wayland client libraries or the Fcitx5 Wayland module are missing: the popup is simply never available.
#include "make.h"
namespace bornopata {
std::unique_ptr<Presenter> makeWaylandPresenter(fcitx::AddonManager *) { return nullptr; }
}
