//! Addon factory.
#include "bornopata.h"
namespace fcitx {
class BornopataFactory final : public AddonFactory {
public:
    AddonInstance *create(AddonManager *manager) override { return new BornopataEngine(manager); }
};
}
FCITX_ADDON_FACTORY_V2(bornopata, fcitx::BornopataFactory)
