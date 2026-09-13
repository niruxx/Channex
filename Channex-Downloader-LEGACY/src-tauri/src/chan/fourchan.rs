use serde::Deserialize;

use super::types::{is_video_ext, MediaItem, ThreadInfo};

#[derive(Deserialize)]
struct FourChanThreadResponse {
    posts: Vec<FourChanPost>,
}

#[derive(Deserialize)]
struct FourChanPost {
    no: u64,
    #[serde(default)]
    sub: Option<String>,
    #[serde(default)]
    com: Option<String>,
    #[serde(default)]
    tim: Option<i64>,
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

pub async fn fetch(
    client: &reqwest::Client,
    board: &str,
    thread_no: u64,
) -> Result<ThreadInfo, String> {
    let api_url = format!("https://a.4cdn.org/{board}/thread/{thread_no}.json");
    let resp = client
        .get(&api_url)
        .send()
        .await
        .map_err(|e| format!("Failed to reach 4chan API: {e}"))?;

    if !resp.status().is_success() {
        return Err(format!(
            "4chan API returned {} — the thread may have 404'd",
            resp.status()
        ));
    }

    let data: FourChanThreadResponse = resp
        .json()
        .await
        .map_err(|e| format!("Failed to parse 4chan thread JSON: {e}"))?;

    let mut media = Vec::new();
    for post in &data.posts {
        let (Some(tim), Some(ext), Some(filename)) =
            (post.tim, post.ext.as_ref(), post.filename.as_ref())
        else {
            continue;
        };
        let media_url = format!("https://i.4cdn.org/{board}/{tim}{ext}");
        let thumb_url = format!("https://i.4cdn.org/{board}/{tim}s.jpg");
        media.push(MediaItem {
            id: format!("4chan-{}-{}", post.no, tim),
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

    let subject = data
        .posts
        .first()
        .and_then(|p| p.sub.clone())
        .or_else(|| data.posts.first().and_then(|p| p.com.clone()));

    Ok(ThreadInfo {
        site: "4chan".to_string(),
        origin: "https://boards.4chan.org".to_string(),
        board: board.to_string(),
        thread_no,
        subject: subject.map(|s| strip_html(&s)),
        op_comment: data.posts.first().and_then(|p| p.com.clone()).map(|c| strip_html(&c)),
        reply_count: data.posts.len().saturating_sub(1) as u32,
        media,
    })
}

fn strip_html(input: &str) -> String {
    let mut out = String::with_capacity(input.len());
    let mut in_tag = false;
    for c in input.replace("<br>", "\n").replace("<br/>", "\n").chars() {
        match c {
            '<' => in_tag = true,
            '>' => in_tag = false,
            _ if !in_tag => out.push(c),
            _ => {}
        }
    }
    out.replace("&quot;", "\"")
        .replace("&#039;", "'")
        .replace("&gt;", ">")
        .replace("&lt;", "<")
        .replace("&amp;", "&")
}
