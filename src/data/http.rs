//! The one HTTP client every request goes through (board lists,
//! catalogs, threads, media, captcha, replies).
//!
//! Policy (carried over from the Qt version): HTTP/1.1 only, a
//! `Channex/<version>` user agent, a 20 s inactivity timeout so a
//! stalled request errors instead of spinning forever, and one silent
//! retry after 500 ms for dropped connections and timeouts - never for
//! an HTTP error status, and never for POSTs (that could double-post).

use std::time::Duration;

use thiserror::Error;

const READ_TIMEOUT: Duration = Duration::from_secs(20);
const CONNECT_TIMEOUT: Duration = Duration::from_secs(15);
const RETRY_DELAY: Duration = Duration::from_millis(500);

#[derive(Debug, Error)]
pub enum FetchError {
    #[error("Couldn't connect: {0}")]
    Network(String),
    #[error("The site stopped sending data partway through")]
    Interrupted,
    #[error("The server answered {status} for {url}")]
    Status { status: u16, url: String },
    #[error(
        "The site sent a web page instead of data - it's probably behind an anti-bot check Channex can't pass"
    )]
    NotData,
    #[error("Unexpected response from the site: {0}")]
    Parse(String),
}

impl From<reqwest::Error> for FetchError {
    fn from(err: reqwest::Error) -> Self {
        if let Some(status) = err.status() {
            FetchError::Status {
                status: status.as_u16(),
                url: err.url().map(|u| u.to_string()).unwrap_or_default(),
            }
        } else if err.is_timeout() {
            FetchError::Network("the request timed out".into())
        } else if err.is_body() || err.is_decode() {
            FetchError::Interrupted
        } else {
            // reqwest's top-level message is generic; the source chain
            // says what actually went wrong (DNS, TLS, refused, ...).
            let mut msg = err.to_string();
            let mut source = std::error::Error::source(&err);
            while let Some(s) = source {
                msg = s.to_string();
                source = s.source();
            }
            if msg.contains("close_notify") || msg.contains("unexpected EOF") {
                return FetchError::Interrupted;
            }
            FetchError::Network(msg)
        }
    }
}

impl From<serde_json::Error> for FetchError {
    fn from(err: serde_json::Error) -> Self {
        FetchError::Parse(err.to_string())
    }
}

/// Cheap to clone (shares one connection pool).
#[derive(Debug, Clone)]
pub struct HttpClient {
    inner: reqwest::Client,
}

impl HttpClient {
    pub fn new() -> Result<Self, FetchError> {
        let inner = reqwest::Client::builder()
            .user_agent(concat!("Channex/", env!("CARGO_PKG_VERSION")))
            .redirect(reqwest::redirect::Policy::limited(10))
            .connect_timeout(CONNECT_TIMEOUT)
            .read_timeout(READ_TIMEOUT)
            // LynxChan ties a captcha to the cookie set when it was fetched.
            .cookie_store(true)
            .build()?;
        Ok(Self { inner })
    }

    pub fn reqwest(&self) -> &reqwest::Client {
        &self.inner
    }

    /// GETs a URL's body, retrying once on a transient failure.
    pub async fn get_bytes(&self, url: &str) -> Result<Vec<u8>, FetchError> {
        match self.get_once(url).await {
            Err(err) if is_transient(&err) => {
                tokio::time::sleep(RETRY_DELAY).await;
                self.get_once(url).await.map_err(FetchError::from)
            }
            other => other.map_err(FetchError::from),
        }
    }

    /// GETs a URL and parses it as JSON. An HTML answer (an error page
    /// or an anti-bot interstitial) becomes [`FetchError::NotData`].
    pub async fn get_json(&self, url: &str) -> Result<serde_json::Value, FetchError> {
        parse_json(&self.get_bytes(url).await?)
    }

    async fn get_once(&self, url: &str) -> Result<Vec<u8>, reqwest::Error> {
        let response = self.inner.get(url).send().await?.error_for_status()?;
        Ok(response.bytes().await?.to_vec())
    }
}

fn is_transient(err: &reqwest::Error) -> bool {
    err.status().is_none()
        && (err.is_timeout() || err.is_connect() || err.is_request() || err.is_body())
}

/// JSON body → value, recognising HTML pages for a clearer error.
pub fn parse_json(body: &[u8]) -> Result<serde_json::Value, FetchError> {
    let start = body
        .iter()
        .position(|b| !b.is_ascii_whitespace())
        .unwrap_or(0);
    if body.get(start) == Some(&b'<') {
        return Err(FetchError::NotData);
    }
    Ok(serde_json::from_slice(body)?)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn html_body_is_not_data() {
        assert!(matches!(
            parse_json(b"  <!DOCTYPE html><html>"),
            Err(FetchError::NotData)
        ));
    }

    #[test]
    fn json_body_parses() {
        assert_eq!(parse_json(br#"{"a":1}"#).unwrap()["a"], 1);
    }

    #[test]
    fn bad_json_is_parse_error() {
        assert!(matches!(parse_json(b"{oops"), Err(FetchError::Parse(_))));
    }
}
