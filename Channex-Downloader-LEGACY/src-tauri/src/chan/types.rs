use serde::Serialize;

#[derive(Serialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct MediaItem {
    pub id: String,
    pub post_no: u64,
    pub original_name: String,
    pub ext: String,
    pub media_url: String,
    pub thumb_url: String,
    pub width: Option<u32>,
    pub height: Option<u32>,
    pub size_bytes: Option<u64>,
    pub is_video: bool,
}

#[derive(Serialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ThreadInfo {
    pub site: String,
    pub origin: String,
    pub board: String,
    pub thread_no: u64,
    pub subject: Option<String>,
    pub op_comment: Option<String>,
    pub reply_count: u32,
    pub media: Vec<MediaItem>,
}

pub fn is_video_ext(ext: &str) -> bool {
    matches!(
        ext.trim_start_matches('.').to_lowercase().as_str(),
        "webm" | "mp4" | "mov" | "avi" | "mkv"
    )
}
