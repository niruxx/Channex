use serde::Deserialize;

use super::types::{is_video_ext, MediaItem, ThreadInfo};

#[derive(Deserialize)]
struct LynxFile {
    #[serde(default, rename = "originalName")]
    original_name: Option<String>,
    path: String,
    #[serde(default)]
    thumb: Option<String>,
    #[serde(default)]
    size: Option<u64>,
    #[serde(default)]
    width: Option<u32>,
    #[serde(default)]
    height: Option<u32>,
}

#[derive(Deserialize)]
struct LynxPost {
    #[serde(rename = "postId")]
    post_id: u64,
    #[serde(default)]
    files: Vec<LynxFile>,
}

#[derive(Deserialize)]
struct LynxThreadResponse {
    #[serde(rename = "threadId")]
    thread_id: u64,
    #[serde(default)]
    subject: Option<String>,
    #[serde(default)]
    message: Option<String>,
    #[serde(default)]
    files: Vec<LynxFile>,
    #[serde(default)]
    posts: Vec<LynxPost>,
}

pub async fn fetch(
    client: &reqwest::Client,
    origin: &str,
    board: &str,
    thread_no: u64,
) -> Result<ThreadInfo, String> {
    let api_url = format!("{origin}/{board}/res/{thread_no}.json");
    let resp = client
        .get(&api_url)
        .send()
        .await
        .map_err(|e| format!("Failed to reach {origin}: {e}"))?;

    if !resp.status().is_success() {
        return Err(format!(
            "{origin} returned {} for that thread",
            resp.status()
        ));
    }

    let data: LynxThreadResponse = resp
        .json()
        .await
        .map_err(|e| format!("Unrecognized JSON shape from {origin}: {e}"))?;

    let mut media = Vec::new();
    push_files(&mut media, origin, data.thread_id, &data.files);
    for post in &data.posts {
        push_files(&mut media, origin, post.post_id, &post.files);
    }

    Ok(ThreadInfo {
        site: "lynxchan".to_string(),
        origin: origin.to_string(),
        board: board.to_string(),
        thread_no,
        subject: data.subject,
        op_comment: data.message,
        reply_count: data.posts.len() as u32,
        media,
    })
}

fn push_files(media: &mut Vec<MediaItem>, origin: &str, post_no: u64, files: &[LynxFile]) {
    for (idx, f) in files.iter().enumerate() {
        let ext = f
            .path
            .rsplit('.')
            .next()
            .unwrap_or("bin")
            .to_lowercase();
        let name = f
            .original_name
            .clone()
            .unwrap_or_else(|| f.path.rsplit('/').next().unwrap_or("file").to_string());
        media.push(MediaItem {
            id: format!("lynx-{post_no}-{idx}"),
            post_no,
            original_name: name,
            ext: ext.clone(),
            media_url: absolute_url(origin, &f.path),
            thumb_url: f
                .thumb
                .as_ref()
                .map(|t| absolute_url(origin, t))
                .unwrap_or_else(|| absolute_url(origin, &f.path)),
            width: f.width,
            height: f.height,
            size_bytes: f.size,
            is_video: is_video_ext(&ext),
        });
    }
}

fn absolute_url(origin: &str, path: &str) -> String {
    if path.starts_with("http://") || path.starts_with("https://") {
        path.to_string()
    } else if let Some(stripped) = path.strip_prefix('/') {
        format!("{origin}/{stripped}")
    } else {
        format!("{origin}/{path}")
    }
}
