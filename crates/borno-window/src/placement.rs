//! Where the popup goes relative to the text cursor.

use crate::geometry::Rect;

/// Gap between the cursor rectangle and the popup, in logical pixels.
const GAP: f32 = 6.0;

/// Top-left corner of a popup of `size` (width, height) for the text cursor `cursor`, all in the same coordinates,
/// kept inside `monitor`: below the cursor, flipped above it when there is no room below, pushed in from the sides.
pub fn place(cursor: Rect, size: (f32, f32), monitor: Rect) -> (f32, f32) {
    let (width, height) = size;
    let below = cursor.bottom() + GAP;
    let above = cursor.y - GAP - height;
    let y = if below + height <= monitor.bottom() || above < monitor.y {
        below
    } else {
        above
    };
    let max_x = (monitor.right() - width).max(monitor.x);
    let max_y = (monitor.bottom() - height).max(monitor.y);
    (cursor.x.clamp(monitor.x, max_x), y.clamp(monitor.y, max_y))
}

/// Top-left corner for a popup of `size` the user pinned at `corner`: where they left it, pulled back inside `monitor`
/// when the monitor changed or the popup grew past an edge.
pub fn pinned(corner: (f32, f32), size: (f32, f32), monitor: Rect) -> (f32, f32) {
    let max_x = (monitor.right() - size.0).max(monitor.x);
    let max_y = (monitor.bottom() - size.1).max(monitor.y);
    (
        corner.0.clamp(monitor.x, max_x),
        corner.1.clamp(monitor.y, max_y),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    const MONITOR: Rect = Rect {
        x: 0.0,
        y: 0.0,
        width: 1920.0,
        height: 1080.0,
    };

    #[test]
    fn goes_below_the_cursor_by_default() {
        let cursor = Rect::new(400.0, 300.0, 2.0, 20.0);
        assert_eq!(place(cursor, (200.0, 150.0), MONITOR), (400.0, 326.0));
    }

    #[test]
    fn flips_above_when_there_is_no_room_below() {
        let cursor = Rect::new(400.0, 1000.0, 2.0, 20.0);
        let (x, y) = place(cursor, (200.0, 150.0), MONITOR);
        assert_eq!(x, 400.0);
        assert_eq!(y, 1000.0 - 6.0 - 150.0);
    }

    #[test]
    fn stays_below_when_it_fits_neither_way() {
        let cursor = Rect::new(10.0, 40.0, 2.0, 20.0);
        let (_, y) = place(cursor, (200.0, 150.0), MONITOR);
        assert_eq!(y, 66.0);
    }

    #[test]
    fn is_pushed_in_from_the_right_and_the_left_edges() {
        let right = place(Rect::new(1900.0, 300.0, 2.0, 20.0), (200.0, 100.0), MONITOR);
        assert_eq!(right.0, 1720.0);
        let left = place(Rect::new(-50.0, 300.0, 2.0, 20.0), (200.0, 100.0), MONITOR);
        assert_eq!(left.0, 0.0);
    }

    #[test]
    fn works_on_a_monitor_that_does_not_start_at_the_origin() {
        let second = Rect::new(1920.0, 0.0, 1280.0, 720.0);
        let (x, y) = place(Rect::new(3100.0, 700.0, 2.0, 20.0), (300.0, 100.0), second);
        assert_eq!(x, 2900.0);
        assert_eq!(y, 700.0 - 6.0 - 100.0);
    }

    #[test]
    fn a_popup_larger_than_the_monitor_sits_at_its_corner() {
        let (x, y) = place(Rect::new(10.0, 10.0, 2.0, 20.0), (4000.0, 4000.0), MONITOR);
        assert_eq!((x, y), (0.0, 0.0));
    }

    #[test]
    fn a_pinned_popup_stays_where_it_was_left_unless_it_would_leave_the_monitor() {
        assert_eq!(
            pinned((300.0, 400.0), (200.0, 100.0), MONITOR),
            (300.0, 400.0)
        );
        assert_eq!(
            pinned((1900.0, 1070.0), (200.0, 100.0), MONITOR),
            (1720.0, 980.0)
        );
        assert_eq!(pinned((-30.0, -5.0), (200.0, 100.0), MONITOR), (0.0, 0.0));
    }
}
