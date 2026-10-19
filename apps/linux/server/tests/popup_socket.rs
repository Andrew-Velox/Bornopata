//! The popup channel over the real socket: attaching, a session that names the channel, and what the shell receives.

mod support;

use std::os::unix::net::UnixStream;
use std::sync::Arc;
use std::thread;
use std::time::Duration;

use borno_linux_server::popup::{PopupHub, PopupSink};
use borno_linux_server::{Router, Shared, ipc};
use borno_platform::protocol::{
    Capabilities, ClientMessage, Event, PROTOCOL_VERSION, PopupClientMessage, PopupServerMessage,
    ServerMessage, SessionId, read_message, write_message,
};
use borno_platform::{Config, PanelConfig};
use borno_transliteration::FixedWordSource;
use support::char_key;
use support::shell::Shell;

const TOKEN: u64 = 4242;

fn connect(path: &std::path::Path) -> UnixStream {
    let stream = UnixStream::connect(path).expect("the Server is listening");
    stream
        .set_read_timeout(Some(Duration::from_secs(5)))
        .unwrap();
    stream
}

#[test]
fn a_shell_attaches_a_channel_receives_the_bitmaps_of_its_sessions_and_cannot_attach_midway() {
    let Ok(hub) = PopupHub::start(&PanelConfig::default()) else {
        eprintln!("skipped: no usable font");
        return;
    };
    let directory = std::env::temp_dir().join(format!("borno-popup-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&directory);
    let path = directory.join("bornopata.sock");

    let server_path = path.clone();
    let server_hub = hub.clone();
    let server = thread::spawn(move || {
        let words = FixedWordSource::new([("kotha".to_owned(), vec!["কথা".to_owned()])]);
        let shared = Shared::new(Arc::new(words), None, None);
        let mut router =
            Router::new(Config::default(), shared).with_popup(Some(Box::new(server_hub.clone())));
        ipc::serve_with_popup(server_path, &mut router, Some(server_hub)).unwrap();
    });
    for _ in 0..100 {
        if path.exists() {
            break;
        }
        thread::sleep(Duration::from_millis(20));
    }

    // The channel: Attach is answered, then the Server may use it.
    let mut channel = connect(&path);
    write_message(
        &mut channel,
        &ClientMessage::Attach {
            protocol: PROTOCOL_VERSION,
            token: TOKEN,
        },
    )
    .unwrap();
    let attached: ServerMessage = read_message(&mut channel).unwrap().unwrap();
    assert_eq!(
        attached,
        ServerMessage::Attached {
            protocol: PROTOCOL_VERSION
        }
    );
    let mut writer = channel.try_clone().unwrap();
    let mut shell = Shell::new(channel);
    for _ in 0..100 {
        if hub.attached(TOKEN) {
            break;
        }
        thread::sleep(Duration::from_millis(20));
    }

    // A session that names the channel gets its frames drawn there and is told not to draw.
    let mut main = connect(&path);
    write_message(
        &mut main,
        &ClientMessage::Open {
            session: SessionId(1),
            app: "popup-test".into(),
            protocol: PROTOCOL_VERSION,
            generation: 1,
            context: "ctx1".into(),
            popup: Some(TOKEN),
        },
    )
    .unwrap();
    let _opened: ServerMessage = read_message(&mut main).unwrap().unwrap();
    let mut send = |event: Event| -> ServerMessage {
        write_message(
            &mut main,
            &ClientMessage::Event {
                session: SessionId(1),
                event,
            },
        )
        .unwrap();
        read_message(&mut main).unwrap().unwrap()
    };
    send(Event::Capabilities(Capabilities::default()));
    let ServerMessage::Result { frame, .. } = send(Event::Key {
        event: char_key('k'),
        release: false,
    }) else {
        panic!("expected a Result");
    };
    assert!(frame.native);
    let present = shell.present();
    assert_eq!(present.context, "ctx1");
    assert!(
        shell.fds.pop_front().is_some(),
        "the pool came with its descriptor"
    );

    // The shell reports a scale over the channel and the Server draws again at it.
    write_message(
        &mut writer,
        &PopupClientMessage::Released {
            buffer: present.buffer,
        },
    )
    .unwrap();
    write_message(&mut writer, &PopupClientMessage::Scale { scale_120: 240 }).unwrap();
    assert_eq!(shell.present().scale_120, 240);

    // Committing the word hides the popup.
    for ch in "otha ".chars() {
        send(Event::Key {
            event: char_key(ch),
            release: false,
        });
    }
    loop {
        if shell.next() == Some(PopupServerMessage::Hide) {
            break;
        }
    }

    // A connection that already opened a session is closed when it tries to attach, not converted.
    write_message(
        &mut main,
        &ClientMessage::Attach {
            protocol: PROTOCOL_VERSION,
            token: 1,
        },
    )
    .unwrap();
    assert!(
        read_message::<_, ServerMessage>(&mut main)
            .ok()
            .flatten()
            .is_none()
    );

    // A channel that closes is forgotten.
    drop(writer);
    drop(shell);
    for _ in 0..100 {
        if !hub.attached(TOKEN) {
            break;
        }
        thread::sleep(Duration::from_millis(20));
    }
    assert!(!hub.attached(TOKEN));

    ipc::request_shutdown();
    server.join().unwrap();
    let _ = std::fs::remove_dir_all(&directory);
}
