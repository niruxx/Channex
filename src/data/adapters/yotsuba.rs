//! 4chan's API and the vichan / Lainchan / 8kun family that copies its
//! shape: `boards.json`, `<board>/catalog.json`, `<board>/thread/<id>.json`
//! (or `res/<id>.json` on vichan).

use serde_json::Value;

use super::json::{bool_of, has, i64_of, str_of, u32_of, u64_of};
use crate::data::comment::Comment;
use crate::data::html::decode_text;
use crate::data::http::FetchError;
use crate::data::model::{Board, MediaLayout, Post, PostFile, Site, Thread};

const VIDEO_EXTS: [&str; 3] = [".webm", ".mp4", ".mov"];

pub fn boards_url(site: &Site) -> String {
    format!("{}/boards.json", site.api_origin)
}

pub fn catalog_url(site: &Site, board: &str) -> String {
    format!("{}/{}/catalog.json", site.api_origin, board)
}

pub fn thread_url(site: &Site, board: &str, thread_id: &str) -> String {
    // 4chan serves thread JSON under /thread/; vichan-family engines
    // mirror their HTML paths under /res/.
    let dir = if site.media_layout == MediaLayout::FlatCdn {
        "thread"
    } else {
        "res"
    };
    format!("{}/{}/{}/{}.json", site.api_origin, board, dir, thread_id)
}

pub fn thread_web_url(site: &Site, board: &str, thread_id: &str) -> String {
    if site.media_layout == MediaLayout::FlatCdn {
        format!("{}/{}/thread/{}", site.site_origin, board, thread_id)
    } else {
        format!("{}/{}/res/{}.html", site.site_origin, board, thread_id)
    }
}

/// Full-size and thumbnail URLs for a file stored as `<tim><ext>`.
pub fn media_urls(site: &Site, board: &str, tim: &str, ext: &str) -> (String, String) {
    media_urls_for(site, site.media_layout, board, tim, ext)
}

fn media_urls_for(
    site: &Site,
    layout: MediaLayout,
    board: &str,
    tim: &str,
    ext: &str,
) -> (String, String) {
    let m = &site.media_origin;
    // vichan keeps SVGs as SVG thumbnails; otherwise the site's format.
    let thumb_ext = if ext.eq_ignore_ascii_case(".svg") {
        ".svg"
    } else {
        site.thumb_ext.as_deref().unwrap_or(".jpg")
    };
    match layout {
        MediaLayout::FlatCdn => (
            format!("{m}/{board}/{tim}{ext}"),
            format!("{m}/{board}/{tim}s.jpg"),
        ),
        MediaLayout::FileStore => (
            format!("{m}/file_store/{tim}{ext}"),
            format!("{m}/file_store/thumb/{tim}{thumb_ext}"),
        ),
        MediaLayout::BoardDirs => (
            format!("{m}/{board}/src/{tim}{ext}"),
            format!("{m}/{board}/thumb/{tim}{thumb_ext}"),
        ),
    }
}

/// `boards.json` is a bare array on vichan/8kun and `{"boards": [...]}`
/// on 4chan.
pub fn parse_boards(value: &Value) -> Result<Vec<Board>, FetchError> {
    let list = value
        .as_array()
        .or_else(|| value.get("boards").and_then(Value::as_array))
        .ok_or_else(|| FetchError::Parse("boards.json has no board list".into()))?;

    Ok(list
        .iter()
        .filter_map(|b| {
            let code = if has(b, "board") {
                str_of(b, "board")
            } else {
                str_of(b, "uri")
            };
            if code.is_empty() {
                return None;
            }
            let title = decode_text(&if has(b, "title") {
                str_of(b, "title")
            } else {
                str_of(b, "boardName")
            });
            let description = decode_text(&if has(b, "meta_description") {
                str_of(b, "meta_description")
            } else {
                str_of(b, "subtitle")
            });
            // 4chan: ws_board (1 = worksafe). vichan/8kun: sfw.
            let nsfw = if has(b, "ws_board") {
                !bool_of(b, "ws_board")
            } else if has(b, "sfw") {
                !bool_of(b, "sfw")
            } else {
                false
            };
            Some(Board {
                code,
                title: title.trim().to_owned(),
                description,
                nsfw,
            })
        })
        .collect())
}

/// `catalog.json`: pages of `{"threads": [...]}`, kept in site order.
pub fn parse_catalog(site: &Site, board: &str, value: &Value) -> Result<Vec<Post>, FetchError> {
    let pages = value
        .as_array()
        .ok_or_else(|| FetchError::Parse("catalog.json isn't a list of pages".into()))?;
    Ok(pages
        .iter()
        .filter_map(|page| page.get("threads").and_then(Value::as_array))
        .flatten()
        .map(|raw| parse_post(site, board, raw))
        .collect())
}

/// `thread/<id>.json`: `{"posts": [op, reply, ...]}`.
pub fn parse_thread(site: &Site, board: &str, value: &Value) -> Result<Thread, FetchError> {
    let posts = value
        .get("posts")
        .and_then(Value::as_array)
        .filter(|p| !p.is_empty())
        .ok_or_else(|| FetchError::Parse("the thread has no posts".into()))?;
    let mut posts = posts.iter().map(|raw| parse_post(site, board, raw));
    let op = posts.next().unwrap_or_default();
    Ok(Thread {
        op,
        replies: posts.collect(),
    })
}

pub fn parse_post(site: &Site, board: &str, raw: &Value) -> Post {
    let no = str_of(raw, "no");
    let resto = i64_of(raw, "resto");
    let is_op = resto == 0;

    let mut files = Vec::new();
    if !bool_of(raw, "filedeleted") {
        if let Some(file) = parse_file(site, board, raw) {
            files.push(file);
        }
        // vichan multi-file posts.
        if let Some(extra) = raw.get("extra_files").and_then(Value::as_array) {
            files.extend(extra.iter().filter_map(|f| parse_file(site, board, f)));
        }
    }

    Post {
        thread_id: if is_op { no.clone() } else { resto.to_string() },
        id: no,
        is_op,
        name: decode_text(&str_of(raw, "name")),
        tripcode: str_of(raw, "trip"),
        capcode: str_of(raw, "capcode"),
        subject: decode_text(&str_of(raw, "sub")),
        timestamp: i64_of(raw, "time") * 1000,
        comment: Comment::from_html(&str_of(raw, "com")),
        files,
        sticky: bool_of(raw, "sticky"),
        closed: bool_of(raw, "closed") || bool_of(raw, "locked"),
        archived: bool_of(raw, "archived"),
        country_code: str_of(raw, "country"),
        country_name: str_of(raw, "country_name"),
        reply_count: u32_of(raw, "replies"),
        image_count: u32_of(raw, "images"),
    }
}

fn parse_file(site: &Site, board: &str, raw: &Value) -> Option<PostFile> {
    let tim = str_of(raw, "tim");
    let ext = str_of(raw, "ext");
    // vichan marks a removed file with tim "deleted".
    if tim.is_empty() || ext.is_empty() || tim == "deleted" {
        return None;
    }
    // 8kun marks each file with `fpath`: 1 = the shared file_store,
    // 0 = the older per-board folders.
    let layout = match site.media_layout {
        MediaLayout::FileStore if has(raw, "fpath") && !bool_of(raw, "fpath") => {
            MediaLayout::BoardDirs
        }
        layout => layout,
    };
    let (url, thumb_url) = media_urls_for(site, layout, board, &tim, &ext);
    Some(PostFile {
        url,
        thumb_url,
        name: format!("{}{}", decode_text(&str_of(raw, "filename")), ext),
        is_video: VIDEO_EXTS.contains(&ext.to_ascii_lowercase().as_str()),
        ext,
        width: u32_of(raw, "w"),
        height: u32_of(raw, "h"),
        thumb_width: u32_of(raw, "tn_w"),
        thumb_height: u32_of(raw, "tn_h"),
        size: u64_of(raw, "fsize"),
        spoiler: bool_of(raw, "spoiler"),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::data::sites::preset;

    fn fixture(name: &str) -> Value {
        let path = format!("{}/tests/fixtures/{name}", env!("CARGO_MANIFEST_DIR"));
        serde_json::from_str(&std::fs::read_to_string(path).unwrap()).unwrap()
    }

    #[test]
    fn fourchan_boards() {
        let boards = parse_boards(&fixture("4chan-boards.json")).unwrap();
        assert_eq!(boards.len(), 2);
        assert_eq!(
            boards[0],
            Board {
                code: "g".into(),
                title: "Technology".into(),
                description: "/g/ - Technology".into(),
                nsfw: false
            }
        );
        assert!(boards[1].nsfw, "ws_board 0 means not worksafe");
    }

    #[test]
    fn vichan_boards_array_with_sfw_flag() {
        let boards = parse_boards(&fixture("8kun-boards.json")).unwrap();
        assert_eq!(boards[0].code, "tech");
        assert_eq!(boards[0].title, "Technology", "titles are trimmed");
        assert!(!boards[0].nsfw);
        assert!(boards[1].nsfw);
    }

    #[test]
    fn fourchan_catalog() {
        let site = preset("4chan").unwrap();
        let posts = parse_catalog(&site, "g", &fixture("4chan-catalog.json")).unwrap();
        assert_eq!(posts.len(), 3, "threads from every page, in order");
        let sticky = &posts[0];
        assert!(sticky.sticky && sticky.closed);
        assert_eq!(sticky.subject, "It's the sticky", "entities decoded");
        let p = &posts[1];
        assert_eq!(p.id, "105000001");
        assert_eq!(p.thread_id, "105000001");
        assert!(p.is_op);
        assert_eq!(p.timestamp, 1_700_000_000_000);
        assert_eq!((p.reply_count, p.image_count), (12, 3));
        let f = &p.files[0];
        assert_eq!(f.url, "https://i.4cdn.org/g/1700000000123.webm");
        assert_eq!(f.thumb_url, "https://i.4cdn.org/g/1700000000123s.jpg");
        assert_eq!(f.name, "clip.webm");
        assert!(f.is_video && f.spoiler);
        assert_eq!((f.width, f.height, f.size), (1280, 720, 2048000));
        assert!(posts[2].files.is_empty(), "filedeleted posts have no files");
    }

    #[test]
    fn fourchan_thread() {
        let site = preset("4chan").unwrap();
        let thread = parse_thread(&site, "g", &fixture("4chan-thread.json")).unwrap();
        assert_eq!(thread.op.id, "105000001");
        assert_eq!(thread.replies.len(), 2);
        assert_eq!(thread.replies[0].thread_id, "105000001");
        assert!(!thread.replies[0].is_op);
        assert_eq!(thread.replies[0].display_name(), "Anonymous");
        let backlinks = thread.backlinks();
        assert_eq!(backlinks["105000001"], vec!["105000002", "105000003"]);
        assert_eq!(backlinks["105000002"], vec!["105000003"]);
        assert_eq!(thread.all_files().len(), 2);
    }

    #[test]
    fn vichan_thread_with_string_tim_and_extra_files() {
        let site = preset("lainchan").unwrap();
        let thread = parse_thread(&site, "λ", &fixture("lainchan-thread.json")).unwrap();
        let files = &thread.op.files;
        assert_eq!(files.len(), 2, "main file + extra_files");
        assert_eq!(files[0].url, "https://lainchan.org/λ/src/1600000000001.png");
        assert_eq!(
            files[0].thumb_url,
            "https://lainchan.org/λ/thumb/1600000000001.png"
        );
        assert_eq!(files[1].url, "https://lainchan.org/λ/src/1600000000002.jpg");
        assert!(thread.op.sticky, "\"1\" strings count as true");
        assert_eq!(thread.replies[0].comment.quoted_ids(), vec!["1000"]);
    }

    #[test]
    fn urls_per_layout() {
        let fourchan = preset("4chan").unwrap();
        assert_eq!(
            thread_url(&fourchan, "g", "1"),
            "https://a.4cdn.org/g/thread/1.json"
        );
        assert_eq!(
            thread_web_url(&fourchan, "g", "1"),
            "https://boards.4chan.org/g/thread/1"
        );
        let lain = preset("lainchan").unwrap();
        assert_eq!(
            thread_url(&lain, "sec", "1"),
            "https://lainchan.org/sec/res/1.json"
        );
        assert_eq!(
            thread_web_url(&lain, "sec", "1"),
            "https://lainchan.org/sec/res/1.html"
        );
        let eightkun = preset("8kun").unwrap();
        assert_eq!(
            media_urls(&eightkun, "tech", "abc", ".png"),
            (
                "https://media.8kun.top/file_store/abc.png".into(),
                "https://media.8kun.top/file_store/thumb/abc.jpg".into()
            )
        );
    }

    #[test]
    fn vichan_svg_thumbs_and_deleted_files() {
        let lain = preset("lainchan").unwrap();
        assert_eq!(
            media_urls(&lain, "λ", "1-0", ".svg").1,
            "https://lainchan.org/λ/thumb/1-0.svg"
        );
        let deleted = serde_json::json!({"no": 1, "resto": 0, "tim": "deleted", "ext": ".png"});
        assert!(parse_post(&lain, "λ", &deleted).files.is_empty());
    }

    #[test]
    fn non_list_catalog_is_an_error() {
        let site = preset("4chan").unwrap();
        assert!(parse_catalog(&site, "g", &serde_json::json!({"oops": 1})).is_err());
    }
}
