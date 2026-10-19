//! Pixel format conversion for `wl_shm`.

/// Premultiplied RGBA (what the renderer produces) to premultiplied `ARGB8888` (what `wl_shm` takes): on a
/// little-endian machine that is the byte order B, G, R, A.
pub fn rgba_to_argb8888(rgba: &[u8]) -> Vec<u8> {
    let mut out = Vec::with_capacity(rgba.len());
    for pixel in rgba.chunks_exact(4) {
        out.extend_from_slice(&[pixel[2], pixel[1], pixel[0], pixel[3]]);
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn swaps_red_and_blue_and_keeps_alpha() {
        let rgba = [10, 20, 30, 255, 1, 2, 3, 4];
        assert_eq!(rgba_to_argb8888(&rgba), [30, 20, 10, 255, 3, 2, 1, 4]);
    }

    #[test]
    fn an_empty_bitmap_gives_an_empty_buffer() {
        assert!(rgba_to_argb8888(&[]).is_empty());
    }
}
