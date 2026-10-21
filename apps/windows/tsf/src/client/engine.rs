use std::io::{Read, Write};

use borno_platform::protocol::{
    Capabilities, ClientMessage, CursorRect, Event, KeyEvent, Mode, PROTOCOL_VERSION,
    ServerMessage, SessionId, read_message, write_message,
};

use super::Reply;
use crate::error::ClientError;

/// One session with the Server, on a connection that is already open (a named pipe on Windows, a socket pair in the
/// tests). Every event gets one answer.
pub struct EngineClient<S> {
    stream: S,
    session: SessionId,
}

impl<S: Read + Write> EngineClient<S> {
    /// Opens a session. `app` is the file name of the host program, for the Server's log; `generation` tells this
    /// connection from earlier ones, and `context` names the input context in the Server's replies.
    /// Returns the typing mode the Server starts the session in.
    pub fn open(
        mut stream: S,
        session: SessionId,
        app: &str,
        generation: u64,
        context: &str,
    ) -> Result<(Self, Mode), ClientError> {
        write_message(
            &mut stream,
            &ClientMessage::Open {
                session,
                app: app.to_owned(),
                protocol: PROTOCOL_VERSION,
                generation,
                context: context.to_owned(),
                popup: None,
            },
        )?;
        match read_message(&mut stream)?.ok_or(ClientError::Closed)? {
            ServerMessage::Opened { mode, .. } => Ok((Self { stream, session }, mode)),
            _ => Err(ClientError::Unexpected("expected the session to open")),
        }
    }

    pub fn session(&self) -> SessionId {
        self.session
    }

    /// What the field is: the first event of a session, which the Server insists on.
    pub fn capabilities(&mut self, capabilities: Capabilities) -> Result<Reply, ClientError> {
        self.call(Event::Capabilities(capabilities))
    }

    /// A key press, or its release.
    pub fn key(&mut self, event: KeyEvent, release: bool) -> Result<Reply, ClientError> {
        self.call(Event::Key { event, release })
    }

    /// Where the text cursor is, in physical screen pixels.
    pub fn cursor(&mut self, rect: CursorRect) -> Result<Reply, ClientError> {
        self.call(Event::Cursor(rect))
    }

    pub fn focus(&mut self, focused: bool) -> Result<Reply, ClientError> {
        self.call(Event::Focus { focused })
    }

    /// Drops the word being typed without committing it.
    pub fn reset(&mut self) -> Result<Reply, ClientError> {
        self.call(Event::Reset)
    }

    /// The input method goes away or loses the focus: the answer carries the word being typed as text to commit.
    /// This shell writes the word into the document itself, so it asks for it back.
    pub fn deactivate(&mut self) -> Result<Reply, ClientError> {
        self.call(Event::Deactivate {
            focus_out: true,
            client_preedit: false,
            capability_changed: false,
        })
    }

    /// Closes the session. No answer.
    pub fn close(mut self) -> Result<(), ClientError> {
        write_message(
            &mut self.stream,
            &ClientMessage::Close {
                session: self.session,
            },
        )?;
        Ok(())
    }

    fn call(&mut self, event: Event) -> Result<Reply, ClientError> {
        write_message(
            &mut self.stream,
            &ClientMessage::Event {
                session: self.session,
                event,
            },
        )?;
        match read_message(&mut self.stream)?.ok_or(ClientError::Closed)? {
            ServerMessage::Result {
                outcome,
                commit,
                frame,
                ..
            } => Ok(Reply {
                outcome,
                commit,
                frame,
            }),
            ServerMessage::Ignored { .. } => Err(ClientError::Unexpected(
                "the Server does not know this session",
            )),
            _ => Err(ClientError::Unexpected("expected the result of an event")),
        }
    }
}
