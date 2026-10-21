//! What every Server does before it listens: read the config, start the log, assemble the engine parts. The platform
//! then adds its connections and its candidate window.

mod config;
mod logging;
mod shared;

pub use config::load_config;
pub use logging::start_logging;
pub use shared::build_shared;
