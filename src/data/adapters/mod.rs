//! Per-engine API access. Each engine module has pure `*_url` builders
//! and `parse_*` functions (tested against fixtures in
//! `tests/fixtures/`), and this module dispatches on [`Site::schema`].

mod json;
pub mod lynxchan;
pub mod yotsuba;

use super::http::{FetchError, HttpClient};
use super::model::{Board, Post, Schema, Site, Thread};

/// The site's live board list (unmerged - see
/// [`crate::data::sites::merge_default_boards`]).
pub async fn fetch_boards(client: &HttpClient, site: &Site) -> Result<Vec<Board>, FetchError> {
    match site.schema {
        Schema::Yotsuba => {
            let value = client.get_json(&yotsuba::boards_url(site)).await?;
            yotsuba::parse_boards(&value)
        }
        Schema::Lynxchan => {
            // LynxChan installs differ in which board index they expose.
            let mut last_err = FetchError::Parse("no board list found".into());
            for url in lynxchan::board_list_urls(site) {
                match client
                    .get_json(&url)
                    .await
                    .and_then(|v| lynxchan::parse_boards(&v))
                {
                    Ok(boards) if !boards.is_empty() => return Ok(boards),
                    Ok(_) => {}
                    Err(err) => last_err = err,
                }
            }
            Err(last_err)
        }
    }
}

/// A board's catalog, in the order the site sends it (bump order).
pub async fn fetch_catalog(
    client: &HttpClient,
    site: &Site,
    board: &str,
) -> Result<Vec<Post>, FetchError> {
    match site.schema {
        Schema::Yotsuba => {
            let value = client.get_json(&yotsuba::catalog_url(site, board)).await?;
            yotsuba::parse_catalog(site, board, &value)
        }
        Schema::Lynxchan => {
            let value = client.get_json(&lynxchan::catalog_url(site, board)).await?;
            lynxchan::parse_catalog(site, &value)
        }
    }
}

pub async fn fetch_thread(
    client: &HttpClient,
    site: &Site,
    board: &str,
    thread_id: &str,
) -> Result<Thread, FetchError> {
    match site.schema {
        Schema::Yotsuba => {
            let value = client
                .get_json(&yotsuba::thread_url(site, board, thread_id))
                .await?;
            yotsuba::parse_thread(site, board, &value)
        }
        Schema::Lynxchan => {
            let value = client
                .get_json(&lynxchan::thread_url(site, board, thread_id))
                .await?;
            lynxchan::parse_thread(site, thread_id, &value)
        }
    }
}

/// The thread's page on the site itself (open in browser, reply).
pub fn thread_web_url(site: &Site, board: &str, thread_id: &str) -> String {
    match site.schema {
        Schema::Yotsuba => yotsuba::thread_web_url(site, board, thread_id),
        Schema::Lynxchan => lynxchan::thread_web_url(site, board, thread_id),
    }
}
