//! Catalog search and sorting.

use std::borrow::Borrow;

use serde::{Deserialize, Serialize};

use super::model::Post;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum SortMode {
    Replies,
    Images,
    /// Oldest thread first (by creation time).
    Oldest,
    /// The site's own order, which is bump order.
    #[default]
    #[serde(other)]
    Bump,
}

impl SortMode {
    pub const ALL: [SortMode; 4] = [
        SortMode::Bump,
        SortMode::Replies,
        SortMode::Images,
        SortMode::Oldest,
    ];

    pub fn label(self) -> &'static str {
        match self {
            SortMode::Bump => "Latest Bump",
            SortMode::Replies => "Most Replies",
            SortMode::Images => "Most Images",
            SortMode::Oldest => "Oldest",
        }
    }
}

/// Indices into `posts` of the threads matching `query` (case-insensitive,
/// against subject, thread number and comment text), ordered by `mode`
/// with stickies always first.
/// Works on `&[Post]` or anything that borrows as a post (`Rc<Post>`).
pub fn filter_and_sort<P: Borrow<Post>>(posts: &[P], query: &str, mode: SortMode) -> Vec<usize> {
    let needle = query.trim().to_lowercase();
    let mut indices: Vec<usize> = posts
        .iter()
        .enumerate()
        .filter(|(_, p)| needle.is_empty() || matches((*p).borrow(), &needle))
        .map(|(i, _)| i)
        .collect();

    // Stable sort: ties keep the site's (bump) order.
    indices.sort_by(|&a, &b| {
        let (pa, pb): (&Post, &Post) = (posts[a].borrow(), posts[b].borrow());
        pb.sticky.cmp(&pa.sticky).then_with(|| match mode {
            SortMode::Bump => std::cmp::Ordering::Equal,
            SortMode::Replies => pb.reply_count.cmp(&pa.reply_count),
            SortMode::Images => pb.image_count.cmp(&pa.image_count),
            SortMode::Oldest => pa.timestamp.cmp(&pb.timestamp),
        })
    });
    indices
}

fn matches(post: &Post, needle: &str) -> bool {
    post.subject.to_lowercase().contains(needle)
        || post.thread_id.contains(needle)
        || post.comment.plain_text().to_lowercase().contains(needle)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::data::comment::Comment;

    fn post(id: &str, subject: &str, replies: u32, time: i64, sticky: bool) -> Post {
        Post {
            id: id.into(),
            thread_id: id.into(),
            subject: subject.into(),
            reply_count: replies,
            image_count: replies / 2,
            timestamp: time,
            sticky,
            comment: Comment::from_plain_text(&format!("body of {subject}")),
            ..Default::default()
        }
    }

    fn posts() -> Vec<Post> {
        vec![
            post("1", "Rust thread", 5, 300, false),
            post("2", "Sticky rules", 0, 100, true),
            post("3", "Linux general", 50, 200, false),
            post("4", "rusty tools", 10, 50, false),
        ]
    }

    #[test]
    fn bump_keeps_site_order_with_stickies_first() {
        assert_eq!(filter_and_sort(&posts(), "", SortMode::Bump), [1, 0, 2, 3]);
    }

    #[test]
    fn sort_modes() {
        assert_eq!(
            filter_and_sort(&posts(), "", SortMode::Replies),
            [1, 2, 3, 0]
        );
        assert_eq!(
            filter_and_sort(&posts(), "", SortMode::Images),
            [1, 2, 3, 0]
        );
        assert_eq!(
            filter_and_sort(&posts(), "", SortMode::Oldest),
            [1, 3, 2, 0]
        );
    }

    #[test]
    fn search_is_case_insensitive_and_checks_body_and_number() {
        assert_eq!(filter_and_sort(&posts(), "RUST", SortMode::Bump), [0, 3]);
        assert_eq!(
            filter_and_sort(&posts(), "body of linux", SortMode::Bump),
            [2]
        );
        assert_eq!(filter_and_sort(&posts(), "  4 ", SortMode::Bump), [3]);
    }

    #[test]
    fn sort_mode_json_matches_qt_values() {
        assert_eq!(
            serde_json::to_string(&SortMode::Replies).unwrap(),
            "\"replies\""
        );
        assert_eq!(
            serde_json::from_str::<SortMode>("\"whatever\"").unwrap(),
            SortMode::Bump
        );
    }
}
