//! Drawing the Bangla sample frames with the built-in theme: size, regions, colors and shaping.

use borno_render::sample::{bangl, next_words, shaping};
use borno_render::{FontLibrary, HitTarget, Layout, Rendered, Renderer, Theme};

/// A renderer with the system fonts, or `None` (with a note) when no Bengali font is installed, in which case the
/// glyph checks cannot mean anything.
fn renderer() -> Option<Renderer> {
    let library = FontLibrary::system("en").expect("a UI font is installed");
    if !library
        .families()
        .iter()
        .any(|family| family.to_lowercase().contains("bengali"))
    {
        eprintln!("skipped: no Bengali font installed");
        return None;
    }
    Some(Renderer::new(library))
}

fn pixel(rendered: &Rendered, x: u32, y: u32) -> [u8; 4] {
    let data = rendered.pixmap.data();
    let at = ((y * rendered.pixmap.width() + x) * 4) as usize;
    [data[at], data[at + 1], data[at + 2], data[at + 3]]
}

/// Whether any pixel in the box is close to `target` (premultiplied values compared as drawn).
fn has_color(
    rendered: &Rendered,
    bounds: (u32, u32, u32, u32),
    target: [u8; 3],
    tolerance: i32,
) -> bool {
    let (x0, y0, x1, y1) = bounds;
    (y0..y1).any(|y| {
        (x0..x1).any(|x| {
            let p = pixel(rendered, x, y);
            p[3] == 255
                && (0..3).all(|c| (i32::from(p[c]) - i32::from(target[c])).abs() <= tolerance)
        })
    })
}

#[test]
fn the_bitmap_has_room_for_the_shadow_around_the_card() {
    let Some(mut renderer) = renderer() else {
        return;
    };
    let rendered = renderer
        .render(&shaping(), Layout::Vertical, &Theme::light(), 2.0)
        .unwrap();
    assert!(rendered.content_x > 0 && rendered.content_y > 0);
    assert!(rendered.pixmap.width() > rendered.content_x + rendered.content_width);
    assert!(rendered.pixmap.height() > rendered.content_y + rendered.content_height);
    let (width, height) = rendered.content_size_points();
    assert!((190.0..400.0).contains(&width), "width {width}");
    assert!((100.0..400.0).contains(&height), "height {height}");
    assert_eq!(rendered.next_frame, None);
}

#[test]
fn every_row_has_a_click_region_and_the_regions_do_not_overlap_vertically() {
    let Some(mut renderer) = renderer() else {
        return;
    };
    let frame = shaping();
    let rendered = renderer
        .render(&frame, Layout::Vertical, &Theme::light(), 2.0)
        .unwrap();
    let rows: Vec<_> = rendered
        .hits
        .iter()
        .filter(|hit| matches!(hit.target, HitTarget::Candidate(_)))
        .collect();
    assert_eq!(rows.len(), frame.rows.len());
    for pair in rows.windows(2) {
        assert!(pair[0].y + pair[0].height <= pair[1].y + 1.0);
    }
    let first = rows[0];
    let (x, y) = ((first.x + 10.0) / 2.0, (first.y + first.height / 2.0) / 2.0);
    assert_eq!(rendered.hit_at(x, y), Some(HitTarget::Candidate(0)));
}

#[test]
fn the_highlighted_row_has_the_green_pill_and_a_learned_row_has_the_star() {
    let Some(mut renderer) = renderer() else {
        return;
    };
    let rendered = renderer
        .render(&shaping(), Layout::Vertical, &Theme::light(), 2.0)
        .unwrap();
    let hit = |index: usize| {
        *rendered
            .hits
            .iter()
            .find(|hit| hit.target == HitTarget::Candidate(index))
            .expect("every row has a region")
    };
    let first = hit(0);
    // The click region starts at the word; the pill and the star sit to its left, so take the whole row band.
    let region = (
        rendered.content_x,
        rendered.content_y + first.y as u32,
        rendered.content_x + rendered.content_width,
        rendered.content_y + (first.y + first.height) as u32,
    );
    assert!(has_color(&rendered, region, [0xd0, 0xea, 0xdf], 14), "pill");
    assert!(has_color(&rendered, region, [0xe0, 0x9a, 0x00], 24), "star");
    // The third row is neither highlighted nor learned: no pill, no star.
    let third = hit(2);
    let plain = (
        rendered.content_x,
        rendered.content_y + third.y as u32,
        rendered.content_x + rendered.content_width,
        rendered.content_y + (third.y + third.height) as u32,
    );
    assert!(!has_color(&rendered, plain, [0xe0, 0x9a, 0x00], 24));
    assert!(!has_color(&rendered, plain, [0xd0, 0xea, 0xdf], 10));
}

#[test]
fn the_dark_theme_is_dark_and_the_light_theme_is_light() {
    let Some(mut renderer) = renderer() else {
        return;
    };
    let light = renderer
        .render(&bangl(), Layout::Vertical, &Theme::light(), 2.0)
        .unwrap();
    renderer.forget();
    let dark = renderer
        .render(&bangl(), Layout::Vertical, &Theme::dark(), 2.0)
        .unwrap();
    let corner = |r: &Rendered| pixel(r, r.content_x + 40, r.content_y + 6);
    assert!(corner(&light)[0] > 200, "{:?}", corner(&light));
    assert!(corner(&dark)[0] < 80, "{:?}", corner(&dark));
}

#[test]
fn the_hint_chip_and_the_arrows_of_next_word_suggestions_are_drawn() {
    let Some(mut renderer) = renderer() else {
        return;
    };
    let rendered = renderer
        .render(&next_words(), Layout::Vertical, &Theme::light(), 2.0)
        .unwrap();
    let whole = (0, 0, rendered.pixmap.width(), rendered.pixmap.height());
    assert!(has_color(&rendered, whole, [0xe2, 0xf0, 0xea], 10), "chip");
    assert_eq!(
        rendered
            .hits
            .iter()
            .filter(|hit| matches!(hit.target, HitTarget::Candidate(_)))
            .count(),
        3
    );
}

#[test]
fn bengali_text_is_shaped_not_drawn_letter_by_letter() {
    let Some(mut renderer) = renderer() else {
        return;
    };
    // A conjunct is one glyph that is narrower than its two letters side by side, and the Bengali font is used
    // for it instead of a fallback.
    let conjunct = renderer.measure_points("ক্ষ", 17.0);
    let letters = renderer.measure_points("ক", 17.0) + renderer.measure_points("ষ", 17.0);
    assert!(
        conjunct > 0.0 && conjunct < letters,
        "{conjunct} vs {letters}"
    );
    let theme = Theme::light();
    let families = renderer.trace_families("বাংলা ক্ষমা বিশ্ববিদ্যালয়", &theme);
    assert!(
        families.iter().any(|f| f.contains("Bengali")),
        "{families:?}"
    );
}

#[test]
fn a_horizontal_layout_draws_the_same_frame_wider_and_shorter() {
    let Some(mut renderer) = renderer() else {
        return;
    };
    let vertical = renderer
        .render(&bangl(), Layout::Vertical, &Theme::light(), 2.0)
        .unwrap();
    renderer.forget();
    let horizontal = renderer
        .render(&bangl(), Layout::Horizontal, &Theme::light(), 2.0)
        .unwrap();
    assert!(horizontal.content_width > vertical.content_width);
    assert!(horizontal.content_height < vertical.content_height);
}
