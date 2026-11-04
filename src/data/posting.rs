//! Replying to threads.
//!
//! LynxChan sites accept replies through their API: fetch a captcha
//! image (which also sets a `captchaid` cookie - the HTTP client keeps
//! cookies, so the reply that follows is matched to it), then post a
//! multipart form to `replyThread.js`. Everything else (4chan, vichan
//! sites) is captcha-gated in ways only the site's own page can handle,
//! so the UI hands off to the browser instead.

use super::http::{FetchError, HttpClient, parse_json};
use super::model::Site;

/// The captcha image for a board. `nonce` keeps caches from serving an
/// old one.
pub fn captcha_url(site: &Site, board: &str, nonce: u64) -> String {
    format!(
        "{}/captcha.js?boardUri={}&d={nonce}",
        site.api_origin, board
    )
}

pub fn reply_url(site: &Site) -> String {
    format!("{}/replyThread.js?json=1", site.api_origin)
}

/// What the reply form collects.
#[derive(Debug, Clone, Default)]
pub struct Reply {
    pub board: String,
    pub thread_id: String,
    pub message: String,
    pub name: String,
    /// The "email" field, e.g. `sage`.
    pub options: String,
    pub captcha: String,
    /// Lets the poster delete the post later; one per session.
    pub password: String,
    pub spoiler: bool,
}

/// Posts a reply to a LynxChan site. Returns the new post's id.
/// Never retried: a retry could post twice.
pub async fn post_lynxchan_reply(
    client: &HttpClient,
    site: &Site,
    reply: &Reply,
) -> Result<String, String> {
    let mut form = reqwest::multipart::Form::new()
        .text("boardUri", reply.board.clone())
        .text("threadId", reply.thread_id.clone())
        .text("message", reply.message.clone())
        .text("name", reply.name.clone())
        .text("email", reply.options.clone())
        .text("captcha", reply.captcha.clone())
        .text("password", reply.password.clone());
    if reply.spoiler {
        form = form.text("spoiler", "1");
    }
    let response = client
        .reqwest()
        .post(reply_url(site))
        .multipart(form)
        .send()
        .await
        .map_err(|e| FetchError::from(e).to_string())?;
    let body = response
        .bytes()
        .await
        .map_err(|e| FetchError::from(e).to_string())?;
    parse_reply_response(&body)
}

/// `{"status": "ok", "data": 123}` → `Ok("123")`; any other status →
/// the site's message (`data` is usually a sentence like "Wrong
/// captcha.").
pub fn parse_reply_response(body: &[u8]) -> Result<String, String> {
    let value = parse_json(body).map_err(|e| match e {
        FetchError::NotData => {
            "The site answered with a web page; it may be down or blocking posts".to_owned()
        }
        _ => "The site's reply didn't make sense; its posting API may have changed".to_owned(),
    })?;
    let status = value
        .get("status")
        .and_then(|s| s.as_str())
        .unwrap_or_default();
    let data = value.get("data");
    if status == "ok" {
        return Ok(match data {
            Some(serde_json::Value::Number(n)) => n.to_string(),
            Some(serde_json::Value::String(s)) => s.clone(),
            _ => String::new(),
        });
    }
    let message = data
        .and_then(|d| d.as_str())
        .filter(|s| !s.is_empty())
        .unwrap_or(status);
    Err(if message.is_empty() {
        "The site rejected the reply".to_owned()
    } else {
        message.to_owned()
    })
}

/// A random 8-character password for deleting your own posts.
pub fn session_password(seed: u64) -> String {
    const ALPHABET: &[u8] = b"abcdefghijklmnopqrstuvwxyz0123456789";
    let mut x = seed | 1;
    (0..8)
        .map(|_| {
            // xorshift64
            x ^= x << 13;
            x ^= x >> 7;
            x ^= x << 17;
            ALPHABET[(x % ALPHABET.len() as u64) as usize] as char
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reply_responses() {
        assert_eq!(
            parse_reply_response(br#"{"status":"ok","data":4567}"#),
            Ok("4567".into())
        );
        assert_eq!(
            parse_reply_response(br#"{"status":"error","data":"Wrong captcha."}"#),
            Err("Wrong captcha.".into())
        );
        assert_eq!(
            parse_reply_response(br#"{"status":"bypassable","data":null}"#),
            Err("bypassable".into())
        );
        assert!(
            parse_reply_response(b"<html>")
                .unwrap_err()
                .contains("web page")
        );
    }

    #[test]
    fn urls() {
        let site = Site {
            api_origin: "https://x.org".into(),
            ..Default::default()
        };
        assert_eq!(
            captcha_url(&site, "b", 7),
            "https://x.org/captcha.js?boardUri=b&d=7"
        );
        assert_eq!(reply_url(&site), "https://x.org/replyThread.js?json=1");
    }

    #[test]
    fn passwords() {
        let a = session_password(12345);
        assert_eq!(a.len(), 8);
        assert!(a.chars().all(|c| c.is_ascii_alphanumeric()));
        assert_ne!(a, session_password(54321));
    }
}
