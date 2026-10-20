//! The date and time shortcuts through the Router.

mod support;

use std::sync::Arc;

use borno_learning::FrequencyLearner;
use borno_linux_server::Shared;
use borno_platform::Config;
use borno_transliteration::UpodeshWordSource;
use support::*;

fn rig(shortcuts: bool) -> Rig {
    let shared = Shared::new(
        Arc::new(UpodeshWordSource::new()),
        Some(FrequencyLearner::in_memory()),
        None,
    )
    .with_shortcuts(shortcuts);
    Rig::with(Config::default(), shared)
}

fn candidates(rig: &mut Rig, code: &str) -> Vec<String> {
    let mut reply = None;
    for ch in code.chars() {
        reply = Some(rig.type_char(ch));
    }
    reply
        .unwrap()
        .frame
        .candidates
        .into_iter()
        .map(|candidate| candidate.text)
        .collect()
}

#[test]
fn tarikh_offers_todays_date_in_bangla_digits() {
    let found = candidates(&mut rig(true), "tarikh");
    assert_eq!(found[0], "তারিখ");
    assert!(found[1].chars().any(|ch| ('০'..='৯').contains(&ch)));
    assert!(found[2].contains('/'));
}

#[test]
fn the_config_switches_the_shortcuts_off() {
    let found = candidates(&mut rig(false), "tarikh");
    assert!(found.iter().all(|text| !text.contains('/')));
}
