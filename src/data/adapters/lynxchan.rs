//! The LynxChan engine: `<board>/catalog.json`, `<board>/res/<id>.json`,
//! and one of several board indexes depending on the install.
//!
//! Comments come as `markdown` (already-rendered HTML) with `message`
//! (plain text) as the fallback.

use serde_json::Value;

use super::json::{bool_of, has, str_of, u32_of, u64_of};
use crate::data::comment::Comment;
use crate::data::html::decode_text;
use crate::data::http::FetchError;
use crate::data::model::{Board, Post, PostFile, Site, Thread};

/// Board indexes to try, in order.
pub fn board_list_urls(site: &Site) -> Vec<String> {
    let o = &site.site_origin;
    vec![
        format!("{o}/boards.js?json=1"),
        format!("{o}/boards.json"),
        format!("{o}/index.json"),
    ]
}

pub fn catalog_url(site: &Site, board: &str) -> String {
    format!("{}/{}/catalog.json", site.api_origin, board)
}

pub fn thread_url(site: &Site, board: &str, thread_id: &str) -> String {
    format!("{}/{}/res/{}.json", site.api_origin, board, thread_id)
}

pub fn thread_web_url(site: &Site, board: &str, thread_id: &str) -> String {
    format!("{}/{}/res/{}.html", site.site_origin, board, thread_id)
}

/// Accepts `boards.js?json=1` (`{"boards": [...]}`, or wrapped in
/// `{"data": ...}` on some versions), a bare
/// `boards.json` array, or `index.json`'s `topBoards`.
pub fn parse_boards(value: &Value) -> Result<Vec<Board>, FetchError> {
    let list = value
        .pointer("/data/boards")
        .or_else(|| value.get("boards"))
        .or_else(|| value.get("topBoards"))
        .or(Some(value))
        .and_then(Value::as_array)
        .ok_or_else(|| FetchError::Parse("no board list".into()))?;
    Ok(list
        .iter()
        .filter_map(|b| {
            let code = if has(b, "boardUri") {
                str_of(b, "boardUri")
            } else {
                str_of(b, "uri")
            };
            if code.is_empty() {
                return None;
            }
            let title = if has(b, "boardName") {
                str_of(b, "boardName")
            } else {
                str_of(b, "title")
            };
            let description = if has(b, "boardDescription") {
                str_of(b, "boardDescription")
            } else {
                str_of(b, "subtitle")
            };
            let nsfw = b
                .get("specialSettings")
                .and_then(Value::as_array)
                .is_some_and(|s| !s.iter().any(|x| x.as_str() == Some("sfw")));
            Some(Board {
                code,
                title: decode_text(&title),
                description: decode_text(&description),
                nsfw,
            })
        })
        .collect())
}

pub fn parse_catalog(site: &Site, value: &Value) -> Result<Vec<Post>, FetchError> {
    let list = value
        .as_array()
        .ok_or_else(|| FetchError::Parse("catalog.json isn't a list".into()))?;
    Ok(list
        .iter()
        .map(|raw| {
            let thread_id = str_of(raw, "threadId");
            let thumb = str_of(raw, "thumb");
            let files = if thumb.is_empty() {
                Vec::new()
            } else {
                // The catalog only has the thumbnail; use it for both.
                let url = resolve(site, &thumb);
                vec![PostFile {
                    thumb_url: url.clone(),
                    ext: extension(&thumb),
                    is_video: str_of(raw, "mime").starts_with("video/"),
                    url,
                    ..Default::default()
                }]
            };
            Post {
                id: thread_id.clone(),
                thread_id,
                is_op: true,
                subject: decode_text(&str_of(raw, "subject")),
                timestamp: parse_iso8601_ms(&str_of(raw, "lastBump")).unwrap_or(0),
                comment: comment_of(raw),
                files,
                sticky: bool_of(raw, "pinned"),
                closed: bool_of(raw, "locked"),
                // postCount counts replies only (checked against Endchan;
                // the Qt version subtracted one for the OP, wrongly).
                reply_count: u32_of(raw, "postCount"),
                image_count: u32_of(raw, "fileCount"),
                ..Default::default()
            }
        })
        .collect())
}

pub fn parse_thread(site: &Site, thread_id: &str, value: &Value) -> Result<Thread, FetchError> {
    if !value.is_object() {
        return Err(FetchError::Parse("thread JSON isn't an object".into()));
    }
    let build = |p: &Value, is_op: bool| Post {
        id: if is_op {
            thread_id.to_owned()
        } else {
            str_of(p, "postId")
        },
        thread_id: thread_id.to_owned(),
        is_op,
        name: decode_text(&str_of(p, "name")),
        capcode: str_of(p, "signedRole"),
        subject: if is_op {
            decode_text(&str_of(p, "subject"))
        } else {
            String::new()
        },
        timestamp: parse_iso8601_ms(&str_of(p, "creation")).unwrap_or(0),
        comment: comment_of(p),
        files: p
            .get("files")
            .and_then(Value::as_array)
            .map(|f| f.iter().map(|f| parse_file(site, f)).collect())
            .unwrap_or_default(),
        sticky: is_op && bool_of(p, "pinned"),
        closed: is_op && bool_of(p, "locked"),
        archived: is_op && bool_of(p, "archived"),
        ..Default::default()
    };
    let op = build(value, true);
    let replies = value
        .get("posts")
        .and_then(Value::as_array)
        .map(|r| r.iter().map(|p| build(p, false)).collect())
        .unwrap_or_default();
    Ok(Thread { op, replies })
}

fn parse_file(site: &Site, raw: &Value) -> PostFile {
    let path = str_of(raw, "path");
    let thumb = str_of(raw, "thumb");
    let url = resolve(site, &path);
    PostFile {
        thumb_url: if thumb.is_empty() {
            url.clone()
        } else {
            resolve(site, &thumb)
        },
        url,
        name: decode_text(&str_of(raw, "originalName")),
        ext: extension(&path),
        width: u32_of(raw, "width"),
        height: u32_of(raw, "height"),
        size: u64_of(raw, "size"),
        is_video: str_of(raw, "mime").starts_with("video/"),
        ..Default::default()
    }
}

fn comment_of(raw: &Value) -> Comment {
    let markdown = str_of(raw, "markdown");
    if !markdown.trim().is_empty() {
        Comment::from_html(&markdown)
    } else {
        Comment::from_plain_text(&str_of(raw, "message"))
    }
}

fn resolve(site: &Site, path: &str) -> String {
    if path.starts_with("http://") || path.starts_with("https://") {
        path.to_owned()
    } else {
        format!("{}{}", site.media_origin, path)
    }
}

fn extension(path: &str) -> String {
    path.rsplit_once('.')
        .filter(|(_, e)| !e.contains('/'))
        .map(|(_, e)| format!(".{e}"))
        .unwrap_or_default()
}

/// `2024-03-05T12:34:56.789Z` (or with a `+hh:mm` offset) → Unix ms.
pub fn parse_iso8601_ms(s: &str) -> Option<i64> {
    let s = s.trim();
    let num = |a: usize, b: usize| s.get(a..b)?.parse::<i64>().ok();
    let (year, month, day) = (num(0, 4)?, num(5, 7)?, num(8, 10)?);
    let (hour, minute, second) = (num(11, 13)?, num(14, 16)?, num(17, 19)?);

    let mut rest = &s[19..];
    let mut millis = 0;
    if let Some(frac) = rest.strip_prefix('.') {
        let digits: String = frac.chars().take_while(char::is_ascii_digit).collect();
        rest = &frac[digits.len()..];
        millis = format!("{digits:0<3}")[..3].parse().ok()?;
    }
    let offset_secs = match rest {
        "" | "Z" | "z" => 0,
        tz => {
            let sign = if tz.starts_with('-') { -1 } else { 1 };
            let h: i64 = tz.get(1..3)?.parse().ok()?;
            let m: i64 = tz.get(tz.len() - 2..)?.parse().ok()?;
            sign * (h * 3600 + m * 60)
        }
    };

    // Days since the Unix epoch (Howard Hinnant's days_from_civil).
    let y = if month <= 2 { year - 1 } else { year };
    let era = y.div_euclid(400);
    let yoe = y - era * 400;
    let mp = (month + 9) % 12;
    let doy = (153 * mp + 2) / 5 + day - 1;
    let doe = yoe * 365 + yoe / 4 - yoe / 100 + doy;
    let days = era * 146_097 + doe - 719_468;

    Some(((days * 86_400 + hour * 3600 + minute * 60 + second - offset_secs) * 1000) + millis)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::data::model::Schema;

    fn site() -> Site {
        Site {
            id: "lynx".into(),
            schema: Schema::Lynxchan,
            api_origin: "https://lynx.example".into(),
            site_origin: "https://lynx.example".into(),
            media_origin: "https://lynx.example".into(),
            ..Default::default()
        }
    }

    fn fixture(name: &str) -> Value {
        let path = format!("{}/tests/fixtures/{name}", env!("CARGO_MANIFEST_DIR"));
        serde_json::from_str(&std::fs::read_to_string(path).unwrap()).unwrap()
    }

    #[test]
    fn iso8601() {
        assert_eq!(parse_iso8601_ms("1970-01-01T00:00:00Z"), Some(0));
        assert_eq!(
            parse_iso8601_ms("2023-11-14T22:13:20.000Z"),
            Some(1_700_000_000_000)
        );
        assert_eq!(
            parse_iso8601_ms("2023-11-14T22:13:20.5Z"),
            Some(1_700_000_000_500)
        );
        assert_eq!(
            parse_iso8601_ms("2023-11-15T00:13:20+02:00"),
            Some(1_700_000_000_000)
        );
        assert_eq!(parse_iso8601_ms("not a date"), None);
    }

    #[test]
    fn boards_from_each_index_shape() {
        let js = parse_boards(&serde_json::json!({"status": "ok", "data": {"boards": [
            {"boardUri": "v", "boardName": "Video Games", "boardDescription": "vidya", "specialSettings": ["sfw"]},
            {"boardUri": "b", "boardName": "Random", "specialSettings": []}
        ]}}))
        .unwrap();
        assert_eq!(js.len(), 2);
        assert!(!js[0].nsfw && js[1].nsfw);
        let top = parse_boards(
            &serde_json::json!({"topBoards": [{"boardUri": "a", "boardName": "Anime"}]}),
        )
        .unwrap();
        assert_eq!(top[0].code, "a");
        let arr = parse_boards(&serde_json::json!([{"uri": "x", "title": "X"}])).unwrap();
        assert_eq!(arr[0].title, "X");
    }

    #[test]
    fn catalog() {
        let site = site();
        let posts = parse_catalog(&site, &fixture("lynxchan-catalog.json")).unwrap();
        assert_eq!(posts.len(), 2);
        let p = &posts[0];
        assert_eq!(p.id, "100");
        assert!(p.sticky);
        assert_eq!(p.files[0].thumb_url, "https://lynx.example/.media/t_abc");
        assert_eq!(
            p.comment.plain_text(),
            "first line\nsecond line",
            "falls back to plain message"
        );
        assert_eq!(p.timestamp, 1_700_000_000_000);
        assert_eq!(posts[1].comment.quoted_ids(), vec!["100"]);
    }

    #[test]
    fn thread() {
        let site = site();
        let thread = parse_thread(&site, "100", &fixture("lynxchan-thread.json")).unwrap();
        assert_eq!(thread.op.id, "100");
        assert_eq!(thread.op.subject, "Hello & welcome");
        assert_eq!(
            thread.op.files[0].url,
            "https://lynx.example/.media/abc.png"
        );
        assert_eq!(thread.op.files[0].ext, ".png");
        assert_eq!(thread.replies[0].id, "101");
        assert_eq!(thread.replies[0].capcode, "Board owner");
        assert!(thread.replies[0].files[0].is_video);
        assert_eq!(thread.backlinks()["100"], vec!["101"]);
    }
}
