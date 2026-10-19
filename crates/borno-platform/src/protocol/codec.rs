//! Wire framing: a 4-byte little-endian length followed by a JSON body.

use std::io::{self, Read, Write};

use serde::Serialize;
use serde::de::DeserializeOwned;

/// Largest accepted frame; stops a corrupt length prefix from causing a huge allocation.
const MAX_FRAME: u32 = 16 * 1024 * 1024;

#[derive(Debug, thiserror::Error)]
pub enum CodecError {
    #[error("io: {0}")]
    Io(#[from] io::Error),

    #[error("json: {0}")]
    Json(#[from] serde_json::Error),

    #[error("frame too large: {0} bytes")]
    TooLarge(usize),
}

/// Writes one frame and flushes.
pub fn write_message<W: Write, T: Serialize>(writer: &mut W, value: &T) -> Result<(), CodecError> {
    let body = serde_json::to_vec(value)?;
    let length = u32::try_from(body.len()).map_err(|_| CodecError::TooLarge(body.len()))?;
    if length > MAX_FRAME {
        return Err(CodecError::TooLarge(body.len()));
    }
    writer.write_all(&length.to_le_bytes())?;
    writer.write_all(&body)?;
    writer.flush()?;
    Ok(())
}

/// Reads one frame. `Ok(None)` means the peer closed cleanly at a frame boundary; closing in the middle is an error.
pub fn read_message<R: Read, T: DeserializeOwned>(reader: &mut R) -> Result<Option<T>, CodecError> {
    let mut prefix = [0u8; 4];
    let mut filled = 0;
    while filled < prefix.len() {
        match reader.read(&mut prefix[filled..]) {
            Ok(0) if filled == 0 => return Ok(None),
            Ok(0) => return Err(io::Error::from(io::ErrorKind::UnexpectedEof).into()),
            Ok(count) => filled += count,
            Err(error) if error.kind() == io::ErrorKind::Interrupted => {}
            Err(error) => return Err(error.into()),
        }
    }
    let length = u32::from_le_bytes(prefix);
    if length > MAX_FRAME {
        return Err(CodecError::TooLarge(length as usize));
    }
    let mut body = vec![0u8; length as usize];
    reader.read_exact(&mut body)?;
    Ok(Some(serde_json::from_slice(&body)?))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn round_trips_several_messages_and_ends_cleanly() {
        let mut wire = Vec::new();
        write_message(&mut wire, &"আমি").unwrap();
        write_message(&mut wire, &42u32).unwrap();
        let mut reader = wire.as_slice();
        assert_eq!(
            read_message::<_, String>(&mut reader).unwrap().as_deref(),
            Some("আমি")
        );
        assert_eq!(read_message::<_, u32>(&mut reader).unwrap(), Some(42));
        assert_eq!(read_message::<_, u32>(&mut reader).unwrap(), None);
    }

    #[test]
    fn a_frame_cut_short_is_an_error() {
        let mut wire = Vec::new();
        write_message(&mut wire, &"hello").unwrap();
        wire.truncate(wire.len() - 1);
        assert!(read_message::<_, String>(&mut wire.as_slice()).is_err());
        assert!(read_message::<_, String>(&mut [1u8, 0].as_slice()).is_err());
    }

    #[test]
    fn an_oversized_prefix_is_refused_before_allocating() {
        let wire = u32::MAX.to_le_bytes();
        assert!(matches!(
            read_message::<_, String>(&mut wire.as_slice()),
            Err(CodecError::TooLarge(_))
        ));
    }

    #[test]
    fn invalid_json_is_an_error() {
        let mut wire = 3u32.to_le_bytes().to_vec();
        wire.extend_from_slice(b"{{{");
        assert!(matches!(
            read_message::<_, String>(&mut wire.as_slice()),
            Err(CodecError::Json(_))
        ));
    }
}
