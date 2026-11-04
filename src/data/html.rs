//! Small HTML text helpers.

use scraper::Html;

/// Decodes entities (`&#039;`, `&amp;`, …) and drops any tags, for
/// plain fields like names and subjects.
pub fn decode_text(input: &str) -> String {
    if !input.contains('&') && !input.contains('<') {
        return input.to_owned();
    }
    let fragment = Html::parse_fragment(input);
    fragment.root_element().text().collect()
}

/// Escapes text for Pango markup (and HTML).
pub fn escape_markup(input: &str) -> String {
    let mut out = String::with_capacity(input.len());
    for ch in input.chars() {
        match ch {
            '&' => out.push_str("&amp;"),
            '<' => out.push_str("&lt;"),
            '>' => out.push_str("&gt;"),
            '"' => out.push_str("&quot;"),
            '\'' => out.push_str("&#39;"),
            c => out.push(c),
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn decodes_numeric_and_named_entities() {
        assert_eq!(
            decode_text("It&#039;s &gt;here &amp; there"),
            "It's >here & there"
        );
    }

    #[test]
    fn strips_tags() {
        assert_eq!(decode_text("<b>bold</b> text"), "bold text");
    }

    #[test]
    fn escapes_markup() {
        assert_eq!(escape_markup("a<b>&\"c'"), "a&lt;b&gt;&amp;&quot;c&#39;");
    }
}
