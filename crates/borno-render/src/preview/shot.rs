//! The result of one preview picture.

use std::path::PathBuf;

use super::ShotError;

#[derive(Debug)]
pub struct Shot {
    /// File name: `vertical.png`, `next-words.png` or `vertical-dark.png`.
    pub file: &'static str,

    /// The written path, or why this picture failed.
    pub result: Result<PathBuf, ShotError>,
}
