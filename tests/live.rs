//! Checks the adapters against the real sites. Needs network access and
//! depends on those sites being up, so these are ignored by default:
//!
//!     cargo test --test live -- --ignored --nocapture

use channex::data::adapters::{fetch_boards, fetch_catalog, fetch_thread};
use channex::data::comment::Palette;
use channex::data::http::HttpClient;
use channex::data::sites::preset;

/// Boards → a board's catalog → its busiest thread, checking each step
/// returns sensible data.
async fn walk(site_id: &str, board: &str) {
    let client = HttpClient::new().unwrap();
    let site = preset(site_id).unwrap();

    match fetch_boards(&client, &site).await {
        Ok(boards) => println!("{site_id}: {} boards", boards.len()),
        // Lainchan has no board index; its preset's list is the fallback.
        Err(err) => println!(
            "{site_id}: no board index ({err}); using {} built-in boards",
            site.default_boards.len()
        ),
    }

    let catalog = fetch_catalog(&client, &site, board).await.unwrap();
    assert!(!catalog.is_empty(), "{site_id}/{board}: empty catalog");
    assert!(catalog.iter().all(|p| p.is_op && !p.id.is_empty()));
    let with_file = catalog
        .iter()
        .find(|p| !p.files.is_empty())
        .expect("some thread has an image");
    assert!(with_file.files[0].thumb_url.starts_with("https://"));
    println!(
        "{site_id}/{board}: {} threads, e.g. {:?}",
        catalog.len(),
        with_file.title()
    );

    let busiest = catalog
        .iter()
        .filter(|p| !p.sticky)
        .max_by_key(|p| p.reply_count)
        .unwrap();
    let thread = fetch_thread(&client, &site, board, &busiest.thread_id)
        .await
        .unwrap();
    assert_eq!(thread.op.id, busiest.thread_id);
    let quoted = thread.backlinks().len();
    println!(
        "{site_id}/{board}/{}: {} posts, {} files, {} posts quoted",
        busiest.thread_id,
        thread.post_count(),
        thread.all_files().len(),
        quoted
    );
    assert!(thread.post_count() > 1 || busiest.reply_count == 0);

    // Every real comment renders to markup Pango accepts (bad markup
    // would make the GTK label show nothing).
    let palette = Palette {
        greentext: "#789922".into(),
        dead_quote: "#888888".into(),
        spoiler: "#000000".into(),
    };
    for post in thread.posts().chain(catalog.iter()) {
        let markup = post.comment.to_pango(&palette);
        // GtkLabel handles <a> itself and gives Pango the rest.
        let without_links = strip_links(&markup);
        if let Err(err) = gtk::pango::parse_markup(&without_links, '\0') {
            panic!(
                "post {} produced invalid markup ({err}):\n{markup}",
                post.id
            );
        }
    }

    // A thumbnail actually downloads.
    let thumb = client
        .get_bytes(&with_file.files[0].thumb_url)
        .await
        .unwrap();
    assert!(thumb.len() > 100, "thumbnail too small");
}

fn strip_links(markup: &str) -> String {
    let mut out = String::with_capacity(markup.len());
    let mut rest = markup;
    while let Some(start) = rest.find("<a href=\"") {
        out.push_str(&rest[..start]);
        let after = &rest[start..];
        rest = &after[after.find('>').map_or(after.len(), |i| i + 1)..];
    }
    out.push_str(rest);
    out.replace("</a>", "")
}

#[tokio::test]
#[ignore]
async fn fourchan() {
    walk("4chan", "g").await;
}

#[tokio::test]
#[ignore]
async fn lainchan() {
    walk("lainchan", "λ").await;
}

#[tokio::test]
#[ignore]
async fn eightkun_catalog_and_thread() {
    // 8kun's media host is often unreachable, so skip the thumbnail step.
    let client = HttpClient::new().unwrap();
    let site = preset("8kun").unwrap();
    let boards = fetch_boards(&client, &site).await.unwrap();
    assert!(!boards.is_empty());
    let catalog = fetch_catalog(&client, &site, "tech").await.unwrap();
    let thread = fetch_thread(&client, &site, "tech", &catalog[0].thread_id)
        .await
        .unwrap();
    println!(
        "8kun: {} boards, /tech/ {} threads, first thread {} posts",
        boards.len(),
        catalog.len(),
        thread.post_count()
    );
}

/// A LynxChan site that's reachable (8chan.moe isn't): checks the
/// LynxChan adapter end to end, read-only.
fn endchan() -> channex::data::model::Site {
    use channex::data::model::{PostEngine, Schema, Site};
    Site {
        id: "endchan".into(),
        name: "Endchan".into(),
        schema: Schema::Lynxchan,
        api_origin: "https://endchan.net".into(),
        site_origin: "https://endchan.net".into(),
        media_origin: "https://endchan.net".into(),
        post_engine: PostEngine::Lynxchan,
        ..Default::default()
    }
}

#[tokio::test]
#[ignore]
async fn lynxchan_endchan() {
    let client = HttpClient::new().unwrap();
    let site = endchan();
    let boards = fetch_boards(&client, &site).await.unwrap();
    assert!(!boards.is_empty());
    // Endchan's server cuts off large responses (curl too), so use the
    // first board whose catalog arrives whole.
    let mut found = None;
    for board in boards.iter().take(8) {
        match fetch_catalog(&client, &site, &board.code).await {
            Ok(catalog) if !catalog.is_empty() => {
                found = Some((board.code.clone(), catalog));
                break;
            }
            Ok(_) => {}
            Err(err) => println!("endchan /{}/: {err}", board.code),
        }
    }
    let (board, catalog) = found.expect("no Endchan catalog loaded whole");
    let first = catalog.iter().find(|p| !p.sticky).unwrap_or(&catalog[0]);
    let thread = fetch_thread(&client, &site, &board, &first.thread_id)
        .await
        .unwrap();
    println!(
        "endchan: {} boards, /{board}/ {} threads; thread {}: catalog says {} replies, thread has {}",
        boards.len(),
        catalog.len(),
        first.thread_id,
        first.reply_count,
        thread.replies.len()
    );
    if let Some(file) = thread.all_files().first() {
        let bytes = client.get_bytes(&file.thumb_url).await.unwrap();
        assert!(bytes.len() > 100);
    }
    let captcha = client
        .get_bytes(&channex::data::posting::captcha_url(&site, &board, 42))
        .await
        .unwrap();
    assert!(
        captcha.starts_with(&[0xff, 0xd8]) || captcha.starts_with(b"\x89PNG"),
        "captcha isn't an image"
    );
}
