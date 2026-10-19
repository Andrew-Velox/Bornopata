//! The JSON shape is the contract with the C++ plugin; these pin it.

use serde_json::json;

use super::*;

#[test]
fn events_use_the_externally_tagged_shape_the_plugin_sends() {
    let message = ClientMessage::Event {
        session: SessionId(7),
        event: Event::Key {
            event: KeyEvent::new(0x61, Some('a'), KeyModifiers::default()),
            release: false,
        },
    };
    assert_eq!(
        serde_json::to_value(&message).unwrap(),
        json!({"Event": {"session": 7, "event": {"Key": {
            "event": {"virtual_key": 97, "character": "a",
                "modifiers": {"ctrl": false, "shift": false, "alt": false, "win": false, "caps": false}},
            "release": false}}}})
    );
    let reset: ClientMessage =
        serde_json::from_value(json!({"Event": {"session": 7, "event": "Reset"}})).unwrap();
    assert_eq!(
        reset,
        ClientMessage::Event {
            session: SessionId(7),
            event: Event::Reset
        }
    );
}

#[test]
fn capabilities_and_open_parse_from_the_plugin_json() {
    let open: ClientMessage = serde_json::from_value(json!({"Open": {
        "session": 1, "app": "kate", "protocol": 1, "generation": 3, "context": "ab12"}}))
    .unwrap();
    assert!(matches!(open, ClientMessage::Open { generation: 3, .. }));
    let capabilities: ClientMessage = serde_json::from_value(json!({"Event": {"session": 1,
        "event": {"Capabilities": {"sensitive": true, "password": false, "disabled": false}}}}))
    .unwrap();
    assert!(matches!(
        capabilities,
        ClientMessage::Event {
            event: Event::Capabilities(Capabilities {
                sensitive: true,
                ..
            }),
            ..
        }
    ));
}

#[test]
fn a_result_serializes_for_the_plugin() {
    let message = ServerMessage::Result {
        session: SessionId(1),
        outcome: KeyOutcome::Consumed,
        commit: None,
        identity: Identity {
            generation: 2,
            context: "ab".into(),
            revision: 5,
        },
        frame: Frame {
            roman: "ami".into(),
            preedit: "আমি".into(),
            candidates: vec![FrameCandidate {
                text: "আমি".into(),
                source: Source::Direct,
            }],
            ..Frame::default()
        },
    };
    let value = serde_json::to_value(&message).unwrap();
    let result = &value["Result"];
    assert_eq!(result["outcome"], "Consumed");
    assert_eq!(result["commit"], json!(null));
    assert_eq!(result["identity"]["revision"], 5);
    assert_eq!(result["frame"]["preedit"], "আমি");
    assert_eq!(result["frame"]["candidates"][0]["source"], "Direct");
    assert_eq!(result["frame"]["mode"], "Bangla");
    assert_eq!(result["frame"]["suggesting"], false);
}

#[test]
fn with_session_readdresses_every_reply_kind() {
    let ignored = ServerMessage::Ignored {
        session: SessionId(1),
    };
    assert_eq!(
        ignored.with_session(SessionId(9)),
        ServerMessage::Ignored {
            session: SessionId(9)
        }
    );
}

#[test]
fn an_empty_frame_hides_the_panel() {
    assert!(Frame::default().is_empty());
    let typing = Frame {
        roman: "a".into(),
        ..Frame::default()
    };
    assert!(!typing.is_empty());
}

#[test]
fn a_cursor_event_carries_the_rectangle_and_its_frame_of_reference() {
    let message: ClientMessage =
        serde_json::from_value(json!({"Event": {"session": 1, "event": {"Cursor": {
        "x": 30.0, "y": 40.0, "width": 2.0, "height": 18.0, "surface_relative": true}}}}))
        .unwrap();
    assert_eq!(
        message,
        ClientMessage::Event {
            session: SessionId(1),
            event: Event::Cursor(CursorRect {
                x: 30.0,
                y: 40.0,
                width: 2.0,
                height: 18.0,
                surface_relative: true
            })
        }
    );
}

#[test]
fn a_frame_from_an_older_server_is_not_native() {
    let frame: Frame = serde_json::from_value(json!({"roman": "", "preedit": "", "candidates": [],
        "highlight": 0, "suggesting": false, "mode": "Bangla"}))
    .unwrap();
    assert!(!frame.native);
    assert_eq!(PROTOCOL_VERSION, 3);
}
