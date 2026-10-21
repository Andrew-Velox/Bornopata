//! The engine layer end to end: an `EngineClient` on one end of a socket pair, the real Router on the other. Types
//! `ami`, settles the word with Space and checks what the text service would write into the document.
//!
//! It runs on Unix only, because a socket pair is the easiest real duplex stream; on Windows the same client runs on the
//! named pipe, which needs a Windows machine to try.
#![cfg(unix)]

use std::os::unix::net::UnixStream;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc;
use std::thread;

use borno_platform::Config;
use borno_platform::protocol::{Capabilities, KeyEvent, KeyModifiers, KeyOutcome, Mode, SessionId};
use borno_server::ipc::{QUEUE, run_router, serve_stream};
use borno_server::{Router, Shared};
use borno_transliteration::UpodeshWordSource;
use bornopata_tsf::client::{Decision, EngineClient};

const SESSION: SessionId = SessionId(7);

/// A letter key as the text service sends it: the virtual key is the capital letter, the character the typed one.
fn letter(c: char) -> KeyEvent {
    KeyEvent::new(
        u32::from(c.to_ascii_uppercase()),
        Some(c),
        KeyModifiers::default(),
    )
}

/// A key with no character, such as Enter or an arrow.
fn function(code: u32) -> KeyEvent {
    KeyEvent::new(code, None, KeyModifiers::default())
}

/// Runs a Router on a thread and serves `server_end` to it. Returns the thread to wait for and the stop flag.
fn spawn_server(server_end: UnixStream) -> (thread::JoinHandle<()>, Arc<AtomicBool>) {
    let (sender, receiver) = mpsc::sync_channel(QUEUE);
    let stop = Arc::new(AtomicBool::new(false));
    let stopper = stop.clone();
    let router_thread = thread::spawn(move || {
        let shared = Shared::new(Arc::new(UpodeshWordSource::new()), None, None);
        let mut router = Router::new(Config::default(), shared);
        run_router(&mut router, &receiver, &stopper);
    });
    thread::spawn(move || {
        serve_stream(server_end, &sender);
    });
    (router_thread, stop)
}

#[test]
fn the_client_types_a_word_and_settles_it() {
    let (client_end, server_end) = UnixStream::pair().unwrap();
    let (router_thread, stop) = spawn_server(server_end);

    let (mut client, mode) =
        EngineClient::open(client_end, SESSION, "test.exe", 1, "ctx").expect("the session opens");
    assert_eq!(mode, Mode::Bangla);
    client
        .capabilities(Capabilities::default())
        .expect("the capabilities are taken");

    // Typing: every letter is eaten and the word is shown inline.
    let mut last = None;
    for c in "ami".chars() {
        let reply = client.key(letter(c), false).expect("a key round-trips");
        let decision = Decision::from_reply(reply);
        assert!(decision.eat, "{c} must be eaten");
        assert!(decision.write);
        assert!(decision.composing);
        assert_eq!(decision.commit, None);
        last = Some(decision);
    }
    let typing = last.unwrap();
    assert!(!typing.preedit.is_empty(), "the word is shown inline");

    // Space settles the highlighted candidate and a space, and the key itself is eaten.
    let space = client
        .key(
            KeyEvent::new(0x20, Some(' '), KeyModifiers::default()),
            false,
        )
        .unwrap();
    let decision = Decision::from_reply(space);
    assert!(decision.eat);
    assert!(!decision.resend);
    assert!(!decision.composing);
    assert_eq!(decision.preedit, "");
    let text = decision.commit.expect("the word is committed");
    assert!(text.ends_with(' '), "the space follows the word: {text:?}");
    assert!(
        text.trim_end().chars().all(|c| !c.is_ascii()),
        "the committed word is Bangla: {text:?}"
    );

    client.close().unwrap();
    stop.store(true, Ordering::Relaxed);
    router_thread.join().unwrap();
}

#[test]
fn enter_after_a_word_settles_it_and_is_sent_again() {
    let (client_end, server_end) = UnixStream::pair().unwrap();
    let (router_thread, stop) = spawn_server(server_end);
    let (mut client, _) = EngineClient::open(client_end, SESSION, "test.exe", 1, "ctx").unwrap();
    client.capabilities(Capabilities::default()).unwrap();
    for c in "ami".chars() {
        client.key(letter(c), false).unwrap();
    }

    // The Server commits the word and lets Enter through; the text service eats it and sends it again after the text.
    let reply = client.key(function(0x0d), false).unwrap();
    assert_eq!(reply.outcome, KeyOutcome::Passthrough);
    let decision = Decision::from_reply(reply);
    assert!(decision.eat);
    assert!(decision.resend);
    assert!(decision.commit.is_some());
    assert!(!decision.composing);

    // With nothing being typed Enter is the application's.
    let reply = client.key(function(0x0d), false).unwrap();
    let decision = Decision::from_reply(reply);
    assert!(!decision.eat);
    assert!(!decision.write);

    client.close().unwrap();
    stop.store(true, Ordering::Relaxed);
    router_thread.join().unwrap();
}

#[test]
fn the_focus_leaving_hands_the_word_back() {
    let (client_end, server_end) = UnixStream::pair().unwrap();
    let (router_thread, stop) = spawn_server(server_end);
    let (mut client, _) = EngineClient::open(client_end, SESSION, "test.exe", 1, "ctx").unwrap();
    client.capabilities(Capabilities::default()).unwrap();
    for c in "ami".chars() {
        client.key(letter(c), false).unwrap();
    }
    // The text service writes the word itself, so the Server hands it over instead of leaving it to the framework.
    let reply = client.deactivate().unwrap();
    assert!(reply.commit.is_some());
    assert!(reply.frame.roman.is_empty());

    client.close().unwrap();
    stop.store(true, Ordering::Relaxed);
    router_thread.join().unwrap();
}
