//! Pages shown in the window's content `AdwNavigationView`.

mod bookmarks;
mod catalog;
mod downloads;
mod thread;

pub use bookmarks::bookmarks_page;
pub use catalog::CatalogPage;
pub use downloads::downloads_page;
pub use thread::ThreadPage;
