//! The connection to the Server: opened on activation and when needed, retried after [`RECONNECT_INTERVAL`] when it
//! fails; a failed call closes it and the next key opens it again.

use std::time::Instant;

use borno_platform::protocol::Capabilities;

use super::launch;
use super::{RECONNECT_INTERVAL, TextService_Impl};
use crate::client::EngineClient;
use crate::client::pipe::connect_default;
use crate::com::log::log;

impl TextService_Impl {
    /// Connects and opens the session (named after this thread, with the host program's file name). A Server that is not
    /// running is started.
    pub(super) fn connect(&self) {
        let session = crate::com::session_id();
        let context = format!("{:x}", session.0);
        let generation = crate::com::next_generation();
        let app = crate::com::host_app_name();
        let opened = connect_default()
            .map_err(|error| error.to_string())
            .and_then(|stream| {
                EngineClient::open(stream, session, &app, generation, &context)
                    .map_err(|error| error.to_string())
            });
        match opened {
            Ok((mut client, _mode)) => {
                // The Server insists on the facts about the field before anything else.
                if let Err(error) = client.capabilities(Capabilities::default()) {
                    log(&format!(
                        "the Server did not take the capabilities: {error}"
                    ));
                    return;
                }
                *self.engine.borrow_mut() = Some(client);
                self.last_connect_failure.set(None);
                log("connected to the Server");
            }
            Err(error) => {
                self.last_connect_failure.set(Some(Instant::now()));
                log(&format!("cannot connect to the Server: {error}"));
                // The Server is started at login by the installer, but it may have been closed or crashed since. Start it
                // here; the next key finds it.
                launch::launch_server();
            }
        }
    }

    /// Connects when not connected, unless the last attempt failed less than [`RECONNECT_INTERVAL`] ago. Returns
    /// whether there is a connection now.
    pub(super) fn ensure_connected(&self) -> bool {
        if self.engine.borrow().is_some() {
            return true;
        }
        let recently_failed = self
            .last_connect_failure
            .get()
            .is_some_and(|at| at.elapsed() < RECONNECT_INTERVAL);
        if recently_failed {
            return false;
        }
        self.connect();
        let connected = self.engine.borrow().is_some();
        if connected && self.shared.foreground() {
            self.report_focus(true);
        }
        connected
    }

    /// After a failed call: the connection is dropped and the next key makes a new one.
    pub(super) fn disconnect(&self) {
        *self.engine.borrow_mut() = None;
        self.last_connect_failure.set(None);
        self.shared.end_composing();
    }

    /// Tells the Server this thread has (or has not) the keyboard focus; nothing when there is no connection.
    pub(super) fn report_focus(&self, focused: bool) {
        let mut guard = self.engine.borrow_mut();
        let Some(client) = guard.as_mut() else {
            return;
        };
        if let Err(error) = client.focus(focused) {
            log(&format!("could not report the focus: {error}"));
            drop(guard);
            self.disconnect();
        }
    }
}
