use url::Url;

pub struct ParsedThreadUrl {
    pub origin: String,
    pub host: String,
    pub board: String,
    pub thread_no: u64,
}

/// Extracts (origin, host, board, thread_no) from a wide variety of imageboard
/// thread URL shapes:
///   4chan:    https://boards.4chan.org/g/thread/12345678/some-slug
///   lynxchan: https://8chan.moe/tech/res/12345.html
///   vichan:   https://oldschan.example/b/res/6789.html
pub fn parse_thread_url(input: &str) -> Result<ParsedThreadUrl, String> {
    let trimmed = input.trim();
    let url = Url::parse(trimmed).map_err(|e| format!("Not a valid URL: {e}"))?;

    let host = url
        .host_str()
        .ok_or_else(|| "URL has no host".to_string())?
        .to_string();

    let origin = format!(
        "{}://{}{}",
        url.scheme(),
        host,
        url.port().map(|p| format!(":{p}")).unwrap_or_default()
    );

    let segments: Vec<&str> = url
        .path_segments()
        .map(|s| s.filter(|seg| !seg.is_empty()).collect())
        .unwrap_or_default();

    if segments.is_empty() {
        return Err("URL has no path — paste a link to a specific thread".to_string());
    }

    let board = segments[0].to_string();

    // Prefer the segment right after "thread" or "res", falling back to any
    // segment that starts with a run of digits.
    let mut thread_no: Option<u64> = None;
    for (i, seg) in segments.iter().enumerate() {
        if (*seg == "thread" || *seg == "res") && i + 1 < segments.len() {
            if let Some(n) = leading_digits(segments[i + 1]) {
                thread_no = Some(n);
                break;
            }
        }
    }
    if thread_no.is_none() {
        for seg in segments.iter().skip(1) {
            if let Some(n) = leading_digits(seg) {
                thread_no = Some(n);
                break;
            }
        }
    }

    let thread_no =
        thread_no.ok_or_else(|| "Couldn't find a thread number in that URL".to_string())?;

    Ok(ParsedThreadUrl {
        origin,
        host,
        board,
        thread_no,
    })
}

fn leading_digits(segment: &str) -> Option<u64> {
    let digits: String = segment.chars().take_while(|c| c.is_ascii_digit()).collect();
    if digits.is_empty() {
        None
    } else {
        digits.parse().ok()
    }
}
