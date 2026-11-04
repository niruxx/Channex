//! Core data types. JSON field names (camelCase) and enum spellings
//! match what the Qt version saved, so existing custom sites load as-is.

use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use super::comment::Comment;

/// Which API shape a site speaks.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Schema {
    /// 4chan's API and the vichan/Lainchan/8kun family that copies it.
    #[default]
    Yotsuba,
    /// The LynxChan engine.
    Lynxchan,
}

/// Where a Yotsuba-style site keeps its media files.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum MediaLayout {
    /// `<media>/<board>/<tim><ext>` (4chan's CDN).
    FlatCdn,
    /// `<media>/file_store/<tim><ext>` (8kun).
    FileStore,
    /// `<media>/<board>/src/<tim><ext>` (vichan/Lainchan default).
    #[default]
    #[serde(other)]
    BoardDirs,
}

/// How replies are posted.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum PostEngine {
    /// In-app posting through the LynxChan API.
    Lynxchan,
    /// Captcha-gated: hand off to the browser.
    #[default]
    #[serde(other)]
    External,
}

#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct Board {
    pub code: String,
    pub title: String,
    pub description: String,
    pub nsfw: bool,
}

impl Board {
    pub fn new(code: &str, title: &str) -> Self {
        Self {
            code: code.into(),
            title: title.into(),
            ..Default::default()
        }
    }
}

#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct Site {
    pub id: String,
    pub name: String,
    pub schema: Schema,
    pub media_layout: MediaLayout,
    /// Base URL for JSON API requests.
    pub api_origin: String,
    /// Base URL for human-facing pages (thread links, report pages).
    pub site_origin: String,
    /// Base URL for images and videos.
    pub media_origin: String,
    pub accent: String,
    pub is_custom: bool,
    pub nsfw: bool,
    pub post_engine: PostEngine,
    /// Shown when board discovery fails, and merged into what it finds.
    pub default_boards: Vec<Board>,
    pub favicon: String,
    /// Thumbnail file extension on Yotsuba-style sites; `None` means
    /// `.jpg` (4chan, 8kun). vichan installs often use `.png`.
    pub thumb_ext: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Default)]
pub struct PostFile {
    pub url: String,
    pub thumb_url: String,
    /// Original filename including extension.
    pub name: String,
    /// Extension with the leading dot, e.g. `.webm`.
    pub ext: String,
    pub width: u32,
    pub height: u32,
    pub thumb_width: u32,
    pub thumb_height: u32,
    /// Bytes.
    pub size: u64,
    pub is_video: bool,
    pub spoiler: bool,
}

#[derive(Debug, Clone, PartialEq, Default)]
pub struct Post {
    pub id: String,
    pub thread_id: String,
    pub is_op: bool,
    pub name: String,
    pub tripcode: String,
    pub capcode: String,
    pub subject: String,
    /// Creation time, Unix milliseconds.
    pub timestamp: i64,
    pub comment: Comment,
    pub files: Vec<PostFile>,
    pub sticky: bool,
    pub closed: bool,
    pub archived: bool,
    pub country_code: String,
    pub country_name: String,
    /// Catalog only: number of replies in the thread.
    pub reply_count: u32,
    /// Catalog only: number of images in the thread.
    pub image_count: u32,
}

impl Post {
    /// "Anonymous" when the poster left the name empty.
    pub fn display_name(&self) -> &str {
        if self.name.is_empty() {
            "Anonymous"
        } else {
            &self.name
        }
    }

    /// Subject, or "Thread <id>" when there isn't one.
    pub fn title(&self) -> String {
        if self.subject.is_empty() {
            format!("Thread {}", self.thread_id)
        } else {
            self.subject.clone()
        }
    }
}

#[derive(Debug, Clone, PartialEq, Default)]
pub struct Thread {
    pub op: Post,
    pub replies: Vec<Post>,
}

impl Thread {
    /// OP followed by every reply.
    pub fn posts(&self) -> impl Iterator<Item = &Post> {
        std::iter::once(&self.op).chain(self.replies.iter())
    }

    pub fn post_count(&self) -> usize {
        1 + self.replies.len()
    }

    /// Every file in the thread, in post order (lightbox filmstrip,
    /// "download all").
    pub fn all_files(&self) -> Vec<&PostFile> {
        self.posts().flat_map(|p| p.files.iter()).collect()
    }

    /// Post id → ids of the posts in this thread that quote it, in
    /// thread order and without duplicates.
    pub fn backlinks(&self) -> HashMap<String, Vec<String>> {
        let mut map: HashMap<String, Vec<String>> = HashMap::new();
        for post in self.posts() {
            for quoted in post.comment.quoted_ids() {
                let list = map.entry(quoted).or_default();
                if !list.contains(&post.id) {
                    list.push(post.id.clone());
                }
            }
        }
        map
    }

    /// Index of a post in [`Thread::posts`] order.
    pub fn index_of(&self, post_id: &str) -> Option<usize> {
        self.posts().position(|p| p.id == post_id)
    }
}
