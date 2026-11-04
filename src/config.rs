//! Build-time constants.

pub const APP_ID: &str = "io.github.niruxx.Channex";
pub const APP_NAME: &str = "Channex";
pub const VERSION: &str = env!("CARGO_PKG_VERSION");
/// GResource path prefix (matches data/resources.gresource.xml).
pub const RESOURCE_BASE: &str = "/io/github/niruxx/Channex";
