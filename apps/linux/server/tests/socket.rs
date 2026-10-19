//! The real Unix socket: handshake, per-connection session numbers, refusals and shutdown.

mod support;

use std::os::unix::net::UnixStream;
use std::sync::Arc;
use std::thread;
use std::time::Duration;

use borno_learning::FrequencyLearner;
use borno_linux_server::{Router, Shared, ipc};
use borno_platform::Config;
use borno_platform::protocol::{
    Capabilities, ClientMessage, Event, KeyOutcome, PROTOCOL_VERSION, ServerMessage, SessionId,
    read_message, write_message,
};
use borno_transliteration::UpodeshWordSource;
use support::char_key;

fn connect(path: &std::path::Path) -> UnixStream {
    let stream = UnixStream::connect(path).expect("the Server is listening");
    stream
        .set_read_timeout(Some(Duration::from_secs(5)))
        .unwrap();
    stream
}

fn open(stream: &mut UnixStream, session: u64, protocol: u32) {
    write_message(
        stream,
        &ClientMessage::Open {
            session: SessionId(session),
            app: "socket-test".into(),
            protocol,
            generation: 1,
            context: format!("ctx{session}"),
        },
    )
    .unwrap();
}

fn send(stream: &mut UnixStream, session: u64, event: Event) -> Option<ServerMessage> {
    write_message(
        stream,
        &ClientMessage::Event {
            session: SessionId(session),
            event,
        },
    )
    .unwrap();
    read_message(stream).ok().flatten()
}

fn type_char(stream: &mut UnixStream, session: u64, ch: char) -> ServerMessage {
    send(
        stream,
        session,
        Event::Key {
            event: char_key(ch),
            release: false,
        },
    )
    .expect("a reply")
}

#[test]
fn the_socket_serves_isolated_sessions_and_refuses_bad_clients() {
    let directory = std::env::temp_dir().join(format!("borno-socket-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&directory);
    let path = directory.join("bornopata.sock");

    let server_path = path.clone();
    let server = thread::spawn(move || {
        let shared = Shared::new(
            Arc::new(UpodeshWordSource::new()),
            Some(FrequencyLearner::in_memory()),
            None,
        );
        let mut router = Router::new(Config::default(), shared);
        ipc::serve(server_path, &mut router).unwrap();
    });
    for _ in 0..100 {
        if path.exists() {
            break;
        }
        thread::sleep(Duration::from_millis(20));
    }

    // Handshake and typing.
    let mut first = connect(&path);
    open(&mut first, 1, PROTOCOL_VERSION);
    let Some(ServerMessage::Opened {
        session, protocol, ..
    }) = read_message(&mut first).unwrap()
    else {
        panic!("expected Opened");
    };
    assert_eq!((session, protocol), (SessionId(1), PROTOCOL_VERSION));
    send(&mut first, 1, Event::Capabilities(Capabilities::default())).unwrap();
    type_char(&mut first, 1, 'a');
    type_char(&mut first, 1, 'm');
    let ServerMessage::Result { frame, outcome, .. } = type_char(&mut first, 1, 'i') else {
        panic!("expected Result");
    };
    assert_eq!(outcome, KeyOutcome::Consumed);
    assert_eq!(frame.preedit, "আমি");

    // The same local session number on another connection is a different session.
    let mut second = connect(&path);
    open(&mut second, 1, PROTOCOL_VERSION);
    let _opened: Option<ServerMessage> = read_message(&mut second).unwrap();
    send(&mut second, 1, Event::Capabilities(Capabilities::default())).unwrap();
    let ServerMessage::Result { frame, .. } = type_char(&mut second, 1, 'x') else {
        panic!("expected Result");
    };
    assert_eq!(frame.roman, "x");

    // A late event for a session that was never opened gets an explicit reply.
    let ignored = send(&mut second, 7, Event::Reset);
    assert!(matches!(ignored, Some(ServerMessage::Ignored { session }) if session == SessionId(7)));

    // A key before the capabilities closes the connection.
    let mut eager = connect(&path);
    open(&mut eager, 1, PROTOCOL_VERSION);
    let _opened: Option<ServerMessage> = read_message(&mut eager).unwrap();
    assert!(send(&mut eager, 1, Event::Reset).is_none());

    // A wrong protocol version closes the connection.
    let mut old = connect(&path);
    open(&mut old, 1, PROTOCOL_VERSION + 1);
    assert!(
        read_message::<_, ServerMessage>(&mut old)
            .ok()
            .flatten()
            .is_none()
    );

    // Garbage closes the connection and does not hurt the others.
    let mut garbage = connect(&path);
    std::io::Write::write_all(&mut garbage, &[5, 0, 0, 0, b'n', b'o', b'p', b'e', b'!']).unwrap();
    assert!(
        read_message::<_, ServerMessage>(&mut garbage)
            .ok()
            .flatten()
            .is_none()
    );
    let ServerMessage::Result { frame, .. } = type_char(&mut first, 1, ' ') else {
        panic!("expected Result");
    };
    assert!(frame.is_empty());

    // A second Server on the same socket is refused while the first runs.
    assert!(ipc::bind_socket(&path).is_err());

    ipc::request_shutdown();
    server.join().unwrap();
    assert!(!path.exists());
    let _ = std::fs::remove_dir_all(&directory);
}
