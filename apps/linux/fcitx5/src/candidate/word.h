//! A candidate: its text and an annotation. Selecting it goes back to the Server, which commits.
#pragma once
#include <fcitx/candidatelist.h>
#include <functional>
namespace bornopata {
class Word final : public fcitx::CandidateWord {
public:
    Word(std::string text, std::string comment, std::function<void(fcitx::InputContext *)> select)
        : CandidateWord(fcitx::Text(text)), select_(std::move(select)) {
        setPlaceHolder(text.empty());
        setComment(fcitx::Text(std::move(comment)));
    }
    void select(fcitx::InputContext *context) const override { if (!isPlaceHolder()) select_(context); }
private:
    /// The selection, guarded by a check that the frame is still the one shown.
    std::function<void(fcitx::InputContext *)> select_;
};
}
