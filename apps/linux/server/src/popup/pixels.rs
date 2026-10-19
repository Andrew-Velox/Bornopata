//! From the renderer's bitmap to what `wl_shm` takes.

/// Copies the `width` x `height` rectangle at (`x`, `y`) of a premultiplied RGBA bitmap `source` (`source_width`
/// pixels per row) into `target` as premultiplied `ARGB8888`, the byte order B, G, R, A on a little-endian machine,
/// `width * 4` bytes per row. Only the card is copied: the compositor puts the corner of the popup at the caret, so
/// the shadow margin around the card must not be part of it.
pub fn copy_cropped(
    source: &[u8],
    source_width: u32,
    (x, y, width, height): (u32, u32, u32, u32),
    target: &mut [u8],
) {
    let row = width as usize * 4;
    for line in 0..height as usize {
        let from = ((y as usize + line) * source_width as usize + x as usize) * 4;
        let to = line * row;
        for (pixel, out) in source[from..from + row]
            .chunks_exact(4)
            .zip(target[to..to + row].chunks_exact_mut(4))
        {
            out.copy_from_slice(&[pixel[2], pixel[1], pixel[0], pixel[3]]);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_crop_keeps_only_the_rectangle_and_swaps_red_and_blue() {
        // A 3x3 bitmap whose pixel at column c, row r is (r*3+c, 0, 100, 255).
        let mut source = Vec::new();
        for index in 0..9u8 {
            source.extend_from_slice(&[index, 0, 100, 255]);
        }
        let mut target = vec![0u8; 2 * 2 * 4];
        copy_cropped(&source, 3, (1, 1, 2, 2), &mut target);
        assert_eq!(
            target,
            [
                100, 0, 4, 255, 100, 0, 5, 255, 100, 0, 7, 255, 100, 0, 8, 255
            ]
        );
    }

    #[test]
    fn an_empty_rectangle_copies_nothing() {
        let mut target = Vec::new();
        copy_cropped(&[], 0, (0, 0, 0, 0), &mut target);
        assert!(target.is_empty());
    }
}
