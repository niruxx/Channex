mod fourchan;
mod lynxchan;
mod parse;
pub mod types;
mod vichan;

pub use types::ThreadInfo;

pub async fn fetch_thread(client: &reqwest::Client, url_str: &str) -> Result<ThreadInfo, String> {
    let parsed = parse::parse_thread_url(url_str)?;

    if parsed.host.ends_with("4chan.org") || parsed.host.ends_with("4channel.org") {
        return fourchan::fetch(client, &parsed.board, parsed.thread_no).await;
    }

    // Unknown board engine. Most modern *chan forks are either lynxchan
    // (8chan.moe, 8chan.se, ...) or a vichan/OpenIB derivative exposing a
    // 4chan-shaped API. Try lynxchan's schema first — its required
    // `threadId` field makes it fail fast on a mismatched shape — then fall
    // back to the vichan/4chan shape.
    match lynxchan::fetch(client, &parsed.origin, &parsed.board, parsed.thread_no).await {
        Ok(info) => Ok(info),
        Err(lynx_err) => {
            match vichan::fetch(client, &parsed.origin, &parsed.board, parsed.thread_no).await {
                Ok(info) => Ok(info),
                Err(vichan_err) => Err(format!(
                    "Couldn't read a thread from {}.\n- as lynxchan: {lynx_err}\n- as vichan: {vichan_err}",
                    parsed.host
                )),
            }
        }
    }
}
