//! See the crate docs. Module map:
//!
//! - [`model`]: sites, boards, posts, files, threads
//! - [`comment`]: board HTML → a small inline tree (+ Pango markup)
//! - [`http`]: the shared HTTP client and request policy
//! - [`adapters`]: per-engine API parsing (Yotsuba, LynxChan)
//! - [`sites`]: preset sites, board merging, thread-URL resolution
//! - [`catalog`]: catalog search and sort
//! - [`store`]: settings + bookmarks on disk (with Qt-era import)
//! - [`downloads`]: file and folder naming for downloads
//! - [`cache`]: when to clear the image cache
//! - [`posting`]: replying to LynxChan threads

pub mod adapters;
pub mod cache;
pub mod catalog;
pub mod comment;
pub mod downloads;
pub mod html;
pub mod http;
pub mod model;
pub mod posting;
pub mod sites;
pub mod store;
