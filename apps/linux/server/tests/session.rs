//! Learning, next-word suggestions, privacy, lifecycle and identity.

mod support;

use std::sync::Arc;

use borno_learning::FrequencyLearner;
use borno_linux_server::{Router, Shared};
use borno_platform::Config;
use borno_platform::protocol::{
    Capabilities, ClientMessage, Event, KeyEvent, KeyModifiers, KeyOutcome, PROTOCOL_VERSION,
    ServerMessage, SessionId, Source,
};
use borno_transliteration::FixedWordSource;
use support::*;

const KOTHA: &[(&str, &[&str])] = &[("kotha", &["কথা", "কোথা", "কোঠা"])];
const SENTENCE: &[(&str, &[&str])] = &[("ami", &["আমি"]), ("bhat", &["ভাত"])];

/// Types `roman` and returns the frame with its candidates, leaving the word in the buffer.
fn show(rig: &mut Rig, roman: &str) -> borno_platform::protocol::Frame {
    let mut frame = Default::default();
    for ch in roman.chars() {
        frame = rig.type_char(ch).frame;
    }
    frame
}

fn teach_ami_bhat(rig: &mut Rig) {
    for _ in 0..2 {
        rig.type_text("ami bhat ");
        rig.event(Event::Reset);
    }
}

#[test]
fn a_chosen_candidate_leads_next_time() {
    let mut rig = Rig::fixed(KOTHA);
    show(&mut rig, "kotha");
    rig.function(DOWN);
    let picked = rig.type_char(' ').commit.unwrap();
    rig.event(Event::Reset);
    let frame = show(&mut rig, "kotha");
    assert_eq!(frame.candidates[0].text, picked.trim());
    assert_eq!(frame.candidates[0].source, Source::Learned);
}

#[test]
fn next_words_appear_after_a_space_and_tab_takes_one() {
    let mut rig = Rig::fixed(SENTENCE);
    teach_ami_bhat(&mut rig);
    rig.type_text("ami");
    let reply = rig.type_char(' ');
    assert_eq!(reply.commit.as_deref(), Some("আমি "));
    assert!(reply.frame.suggesting);
    assert_eq!(reply.frame.candidates[0].text, "ভাত");
    assert_eq!(reply.frame.candidates[0].source, Source::Next);

    let accepted = rig.function(TAB);
    assert_eq!(accepted.outcome, KeyOutcome::Consumed);
    assert_eq!(accepted.commit.as_deref(), Some("ভাত"));
    assert!(!accepted.frame.suggesting);
}

#[test]
fn any_other_key_dismisses_the_suggestions() {
    let mut rig = Rig::fixed(SENTENCE);
    teach_ami_bhat(&mut rig);
    rig.type_text("ami ");
    let reply = rig.type_char('b');
    assert!(!reply.frame.suggesting);
    assert_eq!(reply.frame.roman, "b");

    rig.event(Event::Reset);
    rig.type_text("ami ");
    let escaped = rig.function(ESCAPE);
    assert_eq!(escaped.outcome, KeyOutcome::Consumed);
    assert!(escaped.frame.is_empty());

    rig.event(Event::Reset);
    rig.type_text("ami ");
    let arrow = rig.function(DOWN);
    assert_eq!(arrow.outcome, KeyOutcome::Passthrough);
    assert!(arrow.frame.is_empty());
}

#[test]
fn clicking_a_candidate_commits_it_but_only_for_the_current_frame() {
    let mut rig = Rig::fixed(KOTHA);
    let frame_reply = {
        show(&mut rig, "kotha");
        rig.function(DOWN)
    };
    let stale = {
        let mut identity = frame_reply.identity.clone();
        identity.revision -= 1;
        identity
    };
    let ignored = rig.router.handle(ClientMessage::Event {
        session: SESSION,
        event: Event::Candidate {
            identity: stale,
            index: 2,
        },
    });
    assert!(matches!(ignored, Some(ServerMessage::Ignored { .. })));

    let third = frame_reply.frame.candidates[2].text.clone();
    let clicked = rig.event(Event::Candidate {
        identity: frame_reply.identity,
        index: 2,
    });
    assert_eq!(clicked.outcome, KeyOutcome::Consumed);
    assert_eq!(clicked.commit.as_deref(), Some(third.as_str()));
    assert!(clicked.frame.is_empty());
}

#[test]
fn clicking_a_next_word_commits_it() {
    let mut rig = Rig::fixed(SENTENCE);
    teach_ami_bhat(&mut rig);
    let shown = {
        rig.type_text("ami");
        rig.type_char(' ')
    };
    let clicked = rig.event(Event::Candidate {
        identity: shown.identity,
        index: 0,
    });
    assert_eq!(clicked.commit.as_deref(), Some("ভাত"));
}

#[test]
fn password_fields_are_left_alone() {
    let mut rig = Rig::scaffold();
    rig.type_text("am");
    rig.event(Event::Capabilities(Capabilities {
        password: true,
        ..Capabilities::default()
    }));
    let reply = rig.type_char('i');
    assert_eq!(reply.outcome, KeyOutcome::Passthrough);
    assert!(reply.frame.is_empty());
}

#[test]
fn sensitive_fields_are_not_learned_from() {
    let mut rig = Rig::fixed(KOTHA);
    rig.event(Event::Capabilities(Capabilities {
        sensitive: true,
        ..Capabilities::default()
    }));
    show(&mut rig, "kotha");
    rig.function(DOWN);
    rig.type_char(' ');
    rig.event(Event::Capabilities(Capabilities::default()));
    rig.event(Event::Reset);
    let frame = show(&mut rig, "kotha");
    assert_eq!(frame.candidates[0].source, Source::Direct);
}

#[test]
fn nothing_is_learned_when_learning_is_off_in_the_config() {
    let mut config = Config::default();
    config.general.learning = false;
    let words =
        FixedWordSource::new([("kotha".to_owned(), vec!["কথা".to_owned(), "কোথা".to_owned()])]);
    let mut rig = Rig::with(config, Shared::new(Arc::new(words), None, None));
    show(&mut rig, "kotha");
    rig.function(DOWN);
    rig.type_char(' ');
    let frame = show(&mut rig, "kotha");
    assert!(frame.candidates.iter().all(|c| c.source != Source::Learned));
}

#[test]
fn losing_focus_commits_the_word_unless_the_framework_already_did() {
    let mut rig = Rig::scaffold();
    rig.type_text("ami");
    let by_framework = rig.event(Event::Deactivate {
        focus_out: true,
        client_preedit: true,
        capability_changed: false,
    });
    assert_eq!(by_framework.commit, None);
    assert!(by_framework.frame.is_empty());

    rig.type_text("ami");
    let switched = rig.event(Event::Deactivate {
        focus_out: false,
        client_preedit: true,
        capability_changed: false,
    });
    assert_eq!(switched.commit.as_deref(), Some("আমি"));

    rig.type_text("ami");
    let without_preedit = rig.event(Event::Deactivate {
        focus_out: true,
        client_preedit: false,
        capability_changed: false,
    });
    assert_eq!(without_preedit.commit.as_deref(), Some("আমি"));
}

#[test]
fn reset_drops_the_word_without_committing() {
    let mut rig = Rig::scaffold();
    rig.type_text("ami");
    let reply = rig.event(Event::Reset);
    assert_eq!(reply.commit, None);
    assert!(reply.frame.is_empty());
}

#[test]
fn the_release_of_a_consumed_press_is_consumed_too() {
    let mut rig = Rig::scaffold();
    let a = char_key('a');
    assert_eq!(rig.key(a).outcome, KeyOutcome::Consumed);
    assert_eq!(rig.release(a).outcome, KeyOutcome::Consumed);
    assert_eq!(rig.release(a).outcome, KeyOutcome::Passthrough);

    rig.event(Event::Reset);
    let enter = KeyEvent::new(ENTER, None, KeyModifiers::default());
    assert_eq!(rig.key(enter).outcome, KeyOutcome::Passthrough);
    assert_eq!(rig.release(enter).outcome, KeyOutcome::Passthrough);
}

#[test]
fn replies_carry_a_growing_revision_and_the_shells_identity() {
    let mut rig = Rig::scaffold();
    let first = rig.type_char('a').identity;
    let second = rig.type_char('m').identity;
    assert_eq!(first.generation, 1);
    assert_eq!(first.context, "ctx");
    assert!(second.revision > first.revision);
}

#[test]
fn unknown_sessions_are_ignored_and_closed_ones_are_gone() {
    let mut rig = Rig::scaffold();
    let unknown = rig.router.handle(ClientMessage::Event {
        session: SessionId(99),
        event: Event::Reset,
    });
    assert!(
        matches!(unknown, Some(ServerMessage::Ignored { session }) if session == SessionId(99))
    );
    assert_eq!(
        rig.router.handle(ClientMessage::Close { session: SESSION }),
        None
    );
    assert_eq!(rig.router.session_count(), 0);
    let late = rig.router.handle(ClientMessage::Event {
        session: SESSION,
        event: Event::Reset,
    });
    assert!(matches!(late, Some(ServerMessage::Ignored { .. })));
}

#[test]
fn a_wrong_protocol_version_is_not_opened() {
    let mut rig = Rig::scaffold();
    let reply = rig.router.handle(ClientMessage::Open {
        session: SessionId(2),
        app: "old".into(),
        protocol: PROTOCOL_VERSION + 1,
        generation: 1,
        context: "other".into(),
        popup: None,
    });
    assert_eq!(reply, None);
    assert_eq!(rig.router.session_count(), 1);
}

#[test]
fn sessions_type_separately_but_learn_together() {
    let mut rig = Rig::fixed(KOTHA);
    rig.router.handle(ClientMessage::Open {
        session: SessionId(2),
        app: "second".into(),
        protocol: PROTOCOL_VERSION,
        generation: 1,
        context: "other".into(),
        popup: None,
    });
    let caps = |router: &mut Router| {
        router.handle(ClientMessage::Event {
            session: SessionId(2),
            event: Event::Capabilities(Capabilities::default()),
        });
    };
    caps(&mut rig.router);

    show(&mut rig, "kotha");
    rig.function(DOWN);
    let picked = rig.type_char(' ').commit.unwrap();

    let key = |router: &mut Router, ch: char| {
        router.handle(ClientMessage::Event {
            session: SessionId(2),
            event: Event::Key {
                event: char_key(ch),
                release: false,
            },
        })
    };
    let mut last = None;
    for ch in "kotha".chars() {
        last = key(&mut rig.router, ch);
    }
    let Some(ServerMessage::Result { frame, .. }) = last else {
        panic!("expected a result");
    };
    assert_eq!(frame.roman, "kotha");
    assert_eq!(frame.candidates[0].text, picked.trim());
    assert_eq!(frame.candidates[0].source, Source::Learned);
}

#[test]
fn learned_data_is_saved_when_the_router_goes_away() {
    let directory = std::env::temp_dir().join(format!("borno-server-save-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&directory);
    let words = || {
        Arc::new(FixedWordSource::new([(
            "kotha".to_owned(),
            vec!["কথা".to_owned(), "কোথা".to_owned()],
        )]))
    };
    {
        let learner = FrequencyLearner::open(&directory).unwrap();
        let mut rig = Rig::with(Config::default(), Shared::new(words(), Some(learner), None));
        show(&mut rig, "kotha");
        rig.function(DOWN);
        rig.type_char(' ');
    }
    let learner = FrequencyLearner::open(&directory).unwrap();
    let mut rig = Rig::with(Config::default(), Shared::new(words(), Some(learner), None));
    let frame = show(&mut rig, "kotha");
    assert_eq!(frame.candidates[0].source, Source::Learned);
    let _ = std::fs::remove_dir_all(&directory);
}

#[test]
fn a_change_of_privacy_drops_the_word_typed_before_it() {
    let mut rig = Rig::scaffold();
    rig.type_text("kotha");
    let reply = rig.event(Event::Capabilities(Capabilities {
        sensitive: true,
        ..Capabilities::default()
    }));
    assert!(reply.frame.is_empty());
    // The same facts again are no boundary.
    rig.type_text("ami");
    let reply = rig.event(Event::Capabilities(Capabilities {
        sensitive: true,
        ..Capabilities::default()
    }));
    assert_eq!(reply.frame.roman, "ami");
}
