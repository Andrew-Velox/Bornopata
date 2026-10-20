//! From the renderer's bitmap to what `wl_shm` takes.

/// A rectangle in pixels: left, top, width, height.
pub type Rect = (u32, u32, u32, u32);

/// What of a bitmap `bitmap` pixels wide and high the popup shows: the card `content` plus the theme's declared
/// overhang `[top, right, bottom, left]` (pixels), never beyond the bitmap. The numbers are fixed per theme, never
/// measured from the pixels: the compositor puts the popup's top-left corner at the caret, so a crop that followed an
/// animation would move the card with every frame.
pub fn popup_rect(bitmap: (u32, u32), (cx, cy, cw, ch): Rect, overhang: [u32; 4]) -> Rect {
    let [top, right, bottom, left] = overhang;
    let left = left.min(cx);
    let top = top.min(cy);
    let right = right.min(bitmap.0.saturating_sub(cx + cw));
    let bottom = bottom.min(bitmap.1.saturating_sub(cy + ch));
    (cx - left, cy - top, left + cw + right, top + ch + bottom)
}

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
    fn the_declared_overhang_is_added_around_the_card_and_never_leaves_the_bitmap() {
        let content = (10, 20, 100, 50);
        // No overhang: the card only, as before.
        assert_eq!(popup_rect((200, 120), content, [0; 4]), (10, 20, 100, 50));
        assert_eq!(
            popup_rect((200, 120), content, [8, 6, 4, 3]),
            (7, 12, 109, 62)
        );
        // More than the bitmap holds is cut to what is there: 10 on the left, 20 above, 90 right, 50 below.
        assert_eq!(
            popup_rect((200, 120), content, [99, 99, 99, 99]),
            (0, 0, 200, 120)
        );
    }

    #[test]
    fn the_rectangle_does_not_depend_on_what_is_drawn() {
        // The same inputs give the same rectangle however the bitmap changes frame to frame.
        let first = popup_rect((300, 200), (24, 30, 200, 100), [30, 0, 10, 12]);
        let second = popup_rect((300, 200), (24, 30, 200, 100), [30, 0, 10, 12]);
        assert_eq!(first, second);
        assert_eq!(first.0, 12);
        assert_eq!(first.1, 0);
    }

    #[test]
    fn an_empty_rectangle_copies_nothing() {
        let mut target = Vec::new();
        copy_cropped(&[], 0, (0, 0, 0, 0), &mut target);
        assert!(target.is_empty());
    }
}
