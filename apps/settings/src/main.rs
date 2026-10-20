//! Bornopata Settings: drawn on bamboo-slip, it reads and writes the same config file as the input method, and the
//! Server starts over with a saved change by itself.
//!
//! - `bornopata-settings`: opens the window on the user's config (`paths::config_path`).
//! - `--config FILE`: reads and writes another file instead, to try things without touching the real settings.

mod i18n;
mod message;
mod page_id;
mod settings;
mod shortcut;
mod system;
mod view;

use crate::i18n::t;
use std::path::PathBuf;

use bamboo_slip::{WindowOptions, run_window};
use borno_platform::paths;

use page_id::PageId;
use settings::Settings;

/// The id the window manager matches against the desktop entry, which is what gives the window its icon and name.
const APP_ID: &str = "bornopata-settings";

fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info")),
        )
        .init();
    let mut config = paths::config_path();
    let mut page = None;
    let mut args = std::env::args().skip(1);
    while let Some(argument) = args.next() {
        match argument.as_str() {
            "--config" => {
                config = args
                    .next()
                    .map(PathBuf::from)
                    .ok_or("--config needs a file")?
            }
            "--page" => {
                let name = args.next().ok_or("--page needs a page name")?;
                page = Some(PageId::from_name(&name).ok_or(
                    "unknown page: use typing, keys, appearance, phrases, learning or about",
                )?);
            }
            "--version" => {
                println!("bornopata-settings {}", env!("CARGO_PKG_VERSION"));
                return Ok(());
            }
            other => return Err(format!("unknown argument: {other}").into()),
        }
    }
    let mut settings = Settings::load(config)?;
    if let Some(page) = page {
        settings.page = page;
    }
    let mut options = WindowOptions::new(t("Bornopata Settings"), (940.0, 680.0));
    options.min_size = (660.0, 460.0);
    options.locale = if i18n::is_bangla() { "bn" } else { "en" }.to_owned();
    options.app_id = Some(APP_ID.to_owned());
    run_window(settings, options)?;
    Ok(())
}
