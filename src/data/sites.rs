//! Built-in sites, custom sites, board-list merging, and resolving a
//! pasted thread link to (site, board, thread).

use url::Url;

use super::model::{Board, MediaLayout, PostEngine, Schema, Site};

/// The built-in sites, in sidebar order.
///
/// 8chan.moe was a preset in the Qt version; it was dropped because its
/// API now sits behind a proof-of-work check meant to keep non-browser
/// clients out.
pub fn presets() -> Vec<Site> {
    let fourchan_boards = [
        ("g", "Technology"),
        ("v", "Video Games"),
        ("a", "Anime & Manga"),
        ("tv", "Television & Film"),
        ("mu", "Music"),
        ("fit", "Fitness"),
        ("ck", "Food & Cooking"),
        ("diy", "Do It Yourself"),
        ("sci", "Science & Math"),
        ("lit", "Literature"),
        ("out", "Outdoors"),
        ("co", "Comics & Cartoons"),
        ("fa", "Fashion"),
        ("int", "International"),
        ("wg", "Wallpapers"),
        ("biz", "Business & Finance"),
    ];
    // Lainchan has no boards.json, so this list is all it gets (taken
    // from the site's own navigation bar; blank titles are boards it
    // doesn't name there).
    let lainchan_boards = [
        ("λ", "Programming"),
        ("Δ", "Do It Yourself"),
        ("Ω", "Technology"),
        ("sec", "Security"),
        ("inter", "Games and Interactive Media"),
        ("lit", "Literature"),
        ("music", "Musical and Audible Media"),
        ("vis", "Visual Media"),
        ("hum", "Humanity"),
        ("drug", "Drugs 3.0"),
        ("zzz", "Consciousness and Dreams"),
        ("layer", "layer"),
        ("culture", ""),
        ("psy", ""),
        ("mega", ""),
        ("q", "Questions and Complaints"),
        ("r", "Random"),
    ];
    let boards = |list: &[(&str, &str)]| {
        list.iter()
            .map(|(c, t)| Board::new(c, t))
            .collect::<Vec<_>>()
    };

    vec![
        Site {
            id: "4chan".into(),
            name: "4chan".into(),
            schema: Schema::Yotsuba,
            media_layout: MediaLayout::FlatCdn,
            api_origin: "https://a.4cdn.org".into(),
            site_origin: "https://boards.4chan.org".into(),
            media_origin: "https://i.4cdn.org".into(),
            accent: "#3fae6a".into(),
            nsfw: false,
            post_engine: PostEngine::External,
            default_boards: boards(&fourchan_boards),
            favicon: "https://s.4cdn.org/image/favicon.ico".into(),
            ..Default::default()
        },
        Site {
            id: "8kun".into(),
            name: "8kun".into(),
            schema: Schema::Yotsuba,
            media_layout: MediaLayout::FileStore,
            api_origin: "https://8kun.top".into(),
            site_origin: "https://8kun.top".into(),
            media_origin: "https://media.8kun.top".into(),
            accent: "#c9a227".into(),
            nsfw: true,
            // 8kun isn't LynxChan; its posting is captcha-gated like 4chan's.
            post_engine: PostEngine::External,
            ..Default::default()
        },
        Site {
            id: "lainchan".into(),
            name: "Lainchan".into(),
            schema: Schema::Yotsuba,
            media_layout: MediaLayout::BoardDirs,
            api_origin: "https://lainchan.org".into(),
            site_origin: "https://lainchan.org".into(),
            media_origin: "https://lainchan.org".into(),
            accent: "#4fc3c7".into(),
            nsfw: false,
            post_engine: PostEngine::External,
            default_boards: boards(&lainchan_boards),
            favicon: "https://lainchan.org/favicon.ico".into(),
            thumb_ext: Some(".png".into()),
            ..Default::default()
        },
    ]
}

/// One preset by id.
pub fn preset(id: &str) -> Option<Site> {
    presets().into_iter().find(|s| s.id == id)
}

/// Presets followed by the user's custom sites.
pub fn all_sites(custom: &[Site]) -> Vec<Site> {
    let mut sites = presets();
    sites.extend(custom.iter().cloned());
    sites
}

/// Adds the site's built-in boards to what discovery found (discovery
/// wins on conflicts), sorted by board code.
pub fn merge_default_boards(mut discovered: Vec<Board>, defaults: &[Board]) -> Vec<Board> {
    for board in defaults {
        if !discovered.iter().any(|b| b.code == board.code) {
            discovered.push(board.clone());
        }
    }
    discovered.sort_by(|a, b| a.code.cmp(&b.code));
    discovered
}

/// What the "Add Site" form collects.
#[derive(Debug, Clone)]
pub struct CustomSiteInput {
    pub name: String,
    /// e.g. `https://example.org` - where the API and pages live.
    pub origin: String,
    /// Optional separate media host; defaults to `origin`.
    pub media_origin: Option<String>,
    pub schema: Schema,
    pub media_layout: MediaLayout,
    pub nsfw: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum CustomSiteError {
    #[error("Enter a name for the site")]
    MissingName,
    #[error("Enter the site's address, starting with https:// or http://")]
    BadOrigin,
}

/// Builds a custom site from the form. `id` should be unique (the UI
/// uses a timestamp, like the Qt version did).
pub fn custom_site(id: String, input: &CustomSiteInput) -> Result<Site, CustomSiteError> {
    let name = input.name.trim();
    if name.is_empty() {
        return Err(CustomSiteError::MissingName);
    }
    let origin = normalize_origin(&input.origin).ok_or(CustomSiteError::BadOrigin)?;
    let media_origin = match input
        .media_origin
        .as_deref()
        .map(str::trim)
        .filter(|m| !m.is_empty())
    {
        Some(m) => normalize_origin(m).ok_or(CustomSiteError::BadOrigin)?,
        None => origin.clone(),
    };
    Ok(Site {
        id,
        name: name.to_owned(),
        schema: input.schema,
        media_layout: input.media_layout,
        favicon: format!("{origin}/favicon.ico"),
        api_origin: origin.clone(),
        site_origin: origin,
        media_origin,
        accent: String::new(),
        is_custom: true,
        nsfw: input.nsfw,
        thumb_ext: None,
        post_engine: match input.schema {
            Schema::Lynxchan => PostEngine::Lynxchan,
            Schema::Yotsuba => PostEngine::External,
        },
        default_boards: Vec::new(),
    })
}

/// `https://example.org/` → `https://example.org`; anything that isn't
/// an http(s) URL with a host → `None`.
fn normalize_origin(input: &str) -> Option<String> {
    let url = Url::parse(input.trim()).ok()?;
    if !matches!(url.scheme(), "http" | "https") || url.host_str().is_none() {
        return None;
    }
    Some(url.as_str().trim_end_matches('/').to_owned())
}

/// Where a thread link points.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ThreadRef {
    pub site_id: String,
    pub board: String,
    pub thread_id: String,
}

/// Resolves a pasted thread URL against known sites:
/// `/<board>/thread/<id>` (4chan) or `/<board>/res/<id>.html` (vichan,
/// LynxChan), on the host of one of `sites`.
pub fn resolve_thread_url(input: &str, sites: &[Site]) -> Option<ThreadRef> {
    let url = Url::parse(input.trim()).ok()?;
    let host = url.host_str()?;
    let segments: Vec<String> = url
        .path_segments()?
        .map(|s| {
            percent_encoding::percent_decode_str(s)
                .decode_utf8_lossy()
                .into_owned()
        })
        .collect();

    let (board, thread_id) = match segments.as_slice() {
        [board, kind, id, ..] if kind == "thread" => (board.clone(), id.clone()),
        [board, kind, file, ..] if kind == "res" => {
            (board.clone(), file.strip_suffix(".html")?.to_owned())
        }
        _ => return None,
    };
    if board.is_empty() || thread_id.is_empty() || !thread_id.chars().all(|c| c.is_ascii_digit()) {
        return None;
    }

    sites
        .iter()
        .find(|s| {
            Url::parse(&s.site_origin)
                .ok()
                .and_then(|o| o.host_str().map(|h| h.eq_ignore_ascii_case(host)))
                .unwrap_or(false)
        })
        .map(|s| ThreadRef {
            site_id: s.id.clone(),
            board,
            thread_id,
        })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn presets_are_complete() {
        let ids: Vec<_> = presets().into_iter().map(|s| s.id).collect();
        assert_eq!(ids, ["4chan", "8kun", "lainchan"]);
        assert!(!preset("lainchan").unwrap().default_boards.is_empty());
    }

    #[test]
    fn merge_keeps_discovered_and_sorts() {
        let discovered = vec![Board::new("v", "Live V"), Board::new("a", "Anime")];
        let defaults = vec![Board::new("v", "Default V"), Board::new("g", "Tech")];
        let merged = merge_default_boards(discovered, &defaults);
        let codes: Vec<_> = merged.iter().map(|b| b.code.as_str()).collect();
        assert_eq!(codes, ["a", "g", "v"]);
        assert_eq!(merged[2].title, "Live V");
    }

    #[test]
    fn resolves_thread_links() {
        let sites = presets();
        assert_eq!(
            resolve_thread_url(
                "https://boards.4chan.org/g/thread/105076684/some-slug#p1",
                &sites
            ),
            Some(ThreadRef {
                site_id: "4chan".into(),
                board: "g".into(),
                thread_id: "105076684".into()
            })
        );
        assert_eq!(
            resolve_thread_url("https://lainchan.org/%CE%BB/res/42742.html#42750", &sites),
            Some(ThreadRef {
                site_id: "lainchan".into(),
                board: "λ".into(),
                thread_id: "42742".into()
            })
        );
        assert_eq!(
            resolve_thread_url("https://example.org/g/thread/1", &sites),
            None
        );
        assert_eq!(
            resolve_thread_url("https://boards.4chan.org/g/catalog", &sites),
            None
        );
        assert_eq!(resolve_thread_url("not a url", &sites), None);
    }

    #[test]
    fn custom_site_validation() {
        let input = CustomSiteInput {
            name: " My Board ".into(),
            origin: "https://example.org/".into(),
            media_origin: None,
            schema: Schema::Lynxchan,
            media_layout: MediaLayout::BoardDirs,
            nsfw: false,
        };
        let site = custom_site("custom-1".into(), &input).unwrap();
        assert_eq!(site.name, "My Board");
        assert_eq!(site.api_origin, "https://example.org");
        assert_eq!(site.media_origin, "https://example.org");
        assert_eq!(site.favicon, "https://example.org/favicon.ico");
        assert_eq!(site.post_engine, PostEngine::Lynxchan);
        assert!(site.is_custom);

        let bad = CustomSiteInput {
            origin: "example.org".into(),
            ..input.clone()
        };
        assert_eq!(
            custom_site("x".into(), &bad),
            Err(CustomSiteError::BadOrigin)
        );
        let unnamed = CustomSiteInput {
            name: "  ".into(),
            ..input
        };
        assert_eq!(
            custom_site("x".into(), &unnamed),
            Err(CustomSiteError::MissingName)
        );
    }
}
