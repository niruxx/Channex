//! Reusable widgets.

mod post;
mod thread_card;
mod thread_row;

pub use post::{PostContext, PostWidget};
pub use thread_card::ThreadCard;
pub use thread_row::ThreadRow;

use std::rc::Rc;

use channex::data::model::Post;

/// How a catalog cell should present its thread.
#[derive(Debug, Clone, Copy, Default)]
pub struct CardOptions {
    pub compact: bool,
    /// Thumbnail hidden behind a blur until clicked.
    pub blurred: bool,
    pub bookmarked: bool,
}

/// "12 replies · 3 images".
pub fn stats_text(post: &Post) -> String {
    let plural = |n: u32, one: &str, many: &str| format!("{n} {}", if n == 1 { one } else { many });
    format!(
        "{} · {}",
        plural(post.reply_count, "reply", "replies"),
        plural(post.image_count, "image", "images")
    )
}

/// Shared by cells: which post they show, for late-arriving thumbnails.
pub type PostRef = Rc<Post>;
