use serde::Deserialize;

use super::types::{is_video_ext, MediaItem, ThreadInfo};

#[derive(Deserialize)]
struct VichanThreadResponse {
    posts: Vec<VichanPost>,
}

#[derive(Deserialize)]
struct VichanPost {
    no: u64,
    #[serde(default)]
    sub: Option<String>,
    #[serde(default)]
    com: Option<String>,
    #[serde(default)]
    tim: Option<serde_json::Value>,
    #[serde(default)]
    filename: Option<String>,
    #[serde(default)]
    ext: Option<String>,
    #[serde(default)]
    w: Option<u32>,
    #[serde(default)]
    h: Option<u32>,
    #[serde(default)]
    fsize: Option<u64>,
}

/// Best-effort support for the many small vichan / OpenIB derived imageboards
/// that expose a 4chan-shaped JSON API at `/{board}/res/{no}.json` but host
/// media under the board itself rather than a dedicated CDN domain.
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
        return Err(format!("{origin} returned {} for that thread", resp.status()));
    }

    let data: VichanThreadResponse = resp
        .json()
        .await
        .map_err(|e| format!("Unrecognized JSON shape from {origin}: {e}"))?;

    let mut media = Vec::new();
    for post in &data.posts {
        let (Some(tim_val), Some(ext), Some(filename)) =
            (post.tim.as_ref(), post.ext.as_ref(), post.filename.as_ref())
        else {
            continue;
        };
        let tim = match tim_val {
            serde_json::Value::Number(n) => n.to_string(),
            serde_json::Value::String(s) => s.clone(),
            _ => continue,
        };
        let media_url = format!("{origin}/{board}/src/{tim}{ext}");
        let thumb_url = format!("{origin}/{board}/thumb/{tim}s.jpg");
        media.push(MediaItem {
            id: format!("vichan-{}-{}", post.no, tim),
            post_no: post.no,
            original_name: format!("{filename}{ext}"),
            ext: ext.trim_start_matches('.').to_string(),
            media_url,
            thumb_url,
            width: post.w,
            height: post.h,
            size_bytes: post.fsize,
            is_video: is_video_ext(ext),
        });
    }

    let subject = data.posts.first().and_then(|p| p.sub.clone());

    Ok(ThreadInfo {
        site: "vichan".to_string(),
        origin: origin.to_string(),
        board: board.to_string(),
        thread_no,
        subject,
        op_comment: data.posts.first().and_then(|p| p.com.clone()),
        reply_count: data.posts.len().saturating_sub(1) as u32,
        media,
    })
}
