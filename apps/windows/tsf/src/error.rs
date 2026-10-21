//! Errors while talking to the Server.

use borno_platform::protocol::CodecError;

#[derive(Debug, thiserror::Error)]
pub enum ClientError {
    /// A frame could not be written or read.
    #[error("codec: {0}")]
    Codec(#[from] CodecError),

    /// The Server closed the connection between two frames.
    #[error("the Server closed the connection")]
    Closed,

    /// The Server answered with something that does not belong to the request.
    #[error("unexpected Server message: {0}")]
    Unexpected(&'static str),
}
