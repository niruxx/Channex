//! Channex's GTK-independent core: imageboard models, HTTP, site
//! adapters, comment parsing, and persisted settings/bookmarks.
//!
//! Nothing in [`data`] touches GTK, so it can be unit-tested on its own
//! and the UI layer (in the binary) only renders what it returns.

pub mod data;
