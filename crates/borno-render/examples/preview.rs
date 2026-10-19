//! Draws the sample frames with a theme into PNG files, for looking at a theme while editing it:
//! `cargo run -p borno-render --example preview -- [THEME_DIR] [OUT_DIR]`.

use std::path::PathBuf;

use borno_render::preview::write_previews;

fn main() {
    let mut args = std::env::args().skip(1);
    let theme = args.next().map_or_else(
        || PathBuf::from(concat!(env!("CARGO_MANIFEST_DIR"), "/themes/bornopata")),
        PathBuf::from,
    );
    let out = args
        .next()
        .map_or_else(|| PathBuf::from("target/theme-preview"), PathBuf::from);
    match write_previews(&theme, &out, |_| Vec::new()) {
        Ok(shots) => {
            for shot in shots {
                match shot.result {
                    Ok(path) => println!("wrote {}", path.display()),
                    Err(error) => eprintln!("{}: {error}", shot.file),
                }
            }
        }
        Err(error) => {
            eprintln!("cannot preview {}: {error}", theme.display());
            std::process::exit(1);
        }
    }
}
