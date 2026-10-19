//! Server to shell on the popup channel.

use serde::{Deserialize, Serialize};

use super::PopupFrame;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum PopupServerMessage {
    /// A new pool of shared memory. The file descriptor of its `memfd` travels with this message as `SCM_RIGHTS`
    /// ancillary data; the shell hands it to `wl_shm.create_pool` and keeps no mapping of its own.
    Pool {
        pool: u64,
        size: u64,
    },

    Present(PopupFrame),

    /// Nothing to show any more: destroy the popup surface.
    Hide,
}
