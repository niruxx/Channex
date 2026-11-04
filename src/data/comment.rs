//! Post comments.
//!
//! Imageboards send comments as HTML. Instead of passing that HTML (or
//! a sanitized copy of it) to the UI, it is parsed once into a small
//! tree of [`Inline`] nodes. Everything else derives from the tree:
//! Pango markup for display ([`Comment::to_pango`]), plain text for
//! search and card excerpts, and quoted post ids for backlinks.
//!
//! Markup understood, by engine:
//!
//! | Meaning | 4chan | vichan / Lainchan | LynxChan |
//! |---|---|---|---|
//! | Quote link | `a.quotelink[href=#p123]` | `a[href=…#123]` | `a.quoteLink` |
//! | Greentext | `span.quote` | `span.quote` | `span.greenText` |
//! | Spoiler | `s` | `span.spoiler` | `span.spoiler` |
//! | Dead quote | `span.deadlink` | | |
//! | Emphasis | `b`, `i`, `u`, `pre`/`code` | same | `span.redText` (shown bold) |

use scraper::{ElementRef, Html, Node};

use super::html::escape_markup;

/// One piece of a comment.
#[derive(Debug, Clone, PartialEq)]
pub enum Inline {
    Text(String),
    LineBreak,
    /// `>>123`: a link to another post. `href` is kept for quotes that
    /// point into a different thread.
    Quote {
        post_id: String,
        href: String,
        text: String,
    },
    /// `>>123` whose target was deleted.
    DeadQuote(String),
    Link {
        url: String,
        children: Vec<Inline>,
    },
    Greentext(Vec<Inline>),
    Spoiler(Vec<Inline>),
    Bold(Vec<Inline>),
    Italic(Vec<Inline>),
    Underline(Vec<Inline>),
    Strike(Vec<Inline>),
    /// Preformatted text (`pre` / `code`), whitespace preserved.
    Code(String),
}

/// Theme colors used when rendering to Pango markup. Quote and URL
/// links aren't listed: GTK colors `<a>` links from the theme itself.
#[derive(Debug, Clone)]
pub struct Palette {
    pub greentext: String,
    pub dead_quote: String,
    /// Spoilers are drawn as a solid bar in this color.
    pub spoiler: String,
}

#[derive(Debug, Clone, PartialEq, Default)]
pub struct Comment {
    pub nodes: Vec<Inline>,
}

impl Comment {
    /// Parses board HTML.
    pub fn from_html(html: &str) -> Self {
        if html.trim().is_empty() {
            return Self::default();
        }
        let fragment = Html::parse_fragment(html);
        let mut nodes = convert_children(fragment.root_element());
        trim_breaks(&mut nodes);
        Self {
            nodes: linkify(trim_around_breaks(merge_text(nodes))),
        }
    }

    /// Wraps plain text (newlines become line breaks).
    pub fn from_plain_text(text: &str) -> Self {
        let mut nodes = Vec::new();
        for (i, line) in text.split('\n').enumerate() {
            if i > 0 {
                nodes.push(Inline::LineBreak);
            }
            if !line.is_empty() {
                nodes.push(Inline::Text(line.to_owned()));
            }
        }
        trim_breaks(&mut nodes);
        Self {
            nodes: linkify(nodes),
        }
    }

    pub fn is_empty(&self) -> bool {
        self.nodes.is_empty()
    }

    /// Ids of every post this comment quotes, in order, without
    /// duplicates.
    pub fn quoted_ids(&self) -> Vec<String> {
        fn walk(nodes: &[Inline], out: &mut Vec<String>) {
            for node in nodes {
                match node {
                    Inline::Quote { post_id, .. } => {
                        if !out.contains(post_id) {
                            out.push(post_id.clone());
                        }
                    }
                    other => {
                        if let Some(children) = children_of(other) {
                            walk(children, out);
                        }
                    }
                }
            }
        }
        let mut out = Vec::new();
        walk(&self.nodes, &mut out);
        out
    }

    /// The comment as plain text, with line breaks.
    pub fn plain_text(&self) -> String {
        let mut out = String::new();
        push_plain(&self.nodes, &mut out);
        out
    }

    /// A single-line preview: whitespace collapsed, cut to at most
    /// `max_chars` characters with an ellipsis.
    pub fn excerpt(&self, max_chars: usize) -> String {
        let collapsed = self
            .plain_text()
            .split_whitespace()
            .collect::<Vec<_>>()
            .join(" ");
        if collapsed.chars().count() <= max_chars {
            return collapsed;
        }
        let mut cut: String = collapsed
            .chars()
            .take(max_chars.saturating_sub(1))
            .collect();
        cut = cut.trim_end().to_owned();
        cut.push('…');
        cut
    }

    /// Pango markup for a `GtkLabel` with `use-markup`. Quotes become
    /// `<a href="quote:<id>">` so the label's `activate-link` handler
    /// can tell them apart from web links.
    pub fn to_pango(&self, palette: &Palette) -> String {
        let mut out = String::new();
        push_pango(&self.nodes, palette, false, &mut out);
        out
    }
}

fn children_of(node: &Inline) -> Option<&Vec<Inline>> {
    match node {
        Inline::Link { children, .. }
        | Inline::Greentext(children)
        | Inline::Spoiler(children)
        | Inline::Bold(children)
        | Inline::Italic(children)
        | Inline::Underline(children)
        | Inline::Strike(children) => Some(children),
        _ => None,
    }
}

// ---------------------------------------------------------------------
// HTML → tree
// ---------------------------------------------------------------------

fn convert_children(element: ElementRef<'_>) -> Vec<Inline> {
    let mut out = Vec::new();
    for child in element.children() {
        match child.value() {
            Node::Text(text) => {
                // HTML semantics: runs of whitespace (newlines included)
                // are one space; engines mark real breaks with <br>.
                let text = collapse_whitespace(text);
                if !text.is_empty() {
                    out.push(Inline::Text(text));
                }
            }
            Node::Element(_) => {
                if let Some(el) = ElementRef::wrap(child) {
                    out.extend(convert_element(el));
                }
            }
            _ => {}
        }
    }
    out
}

fn class_tokens(el: ElementRef<'_>) -> Vec<String> {
    el.value()
        .attr("class")
        .map(|c| c.split_whitespace().map(str::to_ascii_lowercase).collect())
        .unwrap_or_default()
}

fn text_of(el: ElementRef<'_>) -> String {
    el.text().collect()
}

fn convert_element(el: ElementRef<'_>) -> Vec<Inline> {
    let name = el.value().name();
    let classes = class_tokens(el);
    let has = |c: &str| classes.iter().any(|t| t == c);

    match name {
        "br" => vec![Inline::LineBreak],
        "wbr" | "script" | "style" | "img" => vec![],
        "a" => {
            let href = el.value().attr("href").unwrap_or_default();
            let text = text_of(el);
            if has("deadlink") {
                return vec![Inline::DeadQuote(text)];
            }
            let looks_like_quote =
                has("quotelink") || href.starts_with('#') || text.starts_with(">>");
            if looks_like_quote {
                if let Some(post_id) = quote_id(href) {
                    return vec![Inline::Quote {
                        post_id,
                        href: href.to_owned(),
                        text,
                    }];
                }
            }
            let children = convert_children(el);
            if href.is_empty() || href.starts_with("javascript:") {
                children
            } else {
                vec![Inline::Link {
                    url: href.to_owned(),
                    children,
                }]
            }
        }
        "span" if has("deadlink") => vec![Inline::DeadQuote(text_of(el))],
        "span" if has("quote") || has("greentext") => vec![Inline::Greentext(convert_children(el))],
        "span" if has("spoiler") => vec![Inline::Spoiler(convert_children(el))],
        "span" if has("redtext") || has("heading") => vec![Inline::Bold(convert_children(el))],
        // 4chan renders [spoiler] as <s>.
        "s" => vec![Inline::Spoiler(convert_children(el))],
        "del" | "strike" => vec![Inline::Strike(convert_children(el))],
        "b" | "strong" => vec![Inline::Bold(convert_children(el))],
        "i" | "em" => vec![Inline::Italic(convert_children(el))],
        "u" => vec![Inline::Underline(convert_children(el))],
        "pre" | "code" => vec![Inline::Code(text_of(el))],
        "p" | "div" | "blockquote" | "li" | "tr" => {
            let mut children = convert_children(el);
            children.push(Inline::LineBreak);
            children
        }
        _ => convert_children(el),
    }
}

/// The post id a quote link points at: the URL fragment (`#p123`,
/// `#123`, `#q123`) or a trailing `/123`.
fn quote_id(href: &str) -> Option<String> {
    let digits =
        |s: &str| (!s.is_empty() && s.chars().all(|c| c.is_ascii_digit())).then(|| s.to_owned());
    if let Some((_, fragment)) = href.rsplit_once('#') {
        let fragment = fragment.trim_start_matches(['p', 'q']);
        return digits(fragment);
    }
    href.trim_end_matches('/')
        .rsplit('/')
        .next()
        .and_then(digits)
}

/// Drops leading/trailing line breaks (block elements add one each).
fn trim_breaks(nodes: &mut Vec<Inline>) {
    while matches!(nodes.last(), Some(Inline::LineBreak)) {
        nodes.pop();
    }
    while matches!(nodes.first(), Some(Inline::LineBreak)) {
        nodes.remove(0);
    }
}

fn collapse_whitespace(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    let mut last_space = false;
    for ch in text.chars() {
        if ch.is_whitespace() {
            if !last_space {
                out.push(' ');
            }
            last_space = true;
        } else {
            out.push(ch);
            last_space = false;
        }
    }
    out
}

/// Drops the spaces around line breaks (`text <br> more` shouldn't
/// indent "more"), recursively.
fn trim_around_breaks(nodes: Vec<Inline>) -> Vec<Inline> {
    let mut out: Vec<Inline> = Vec::with_capacity(nodes.len());
    for node in nodes {
        let node = map_children(node, trim_around_breaks);
        match node {
            Inline::Text(t) if matches!(out.last(), Some(Inline::LineBreak) | None) => {
                let t = t.trim_start().to_owned();
                if !t.is_empty() {
                    out.push(Inline::Text(t));
                }
            }
            Inline::LineBreak => {
                if let Some(Inline::Text(prev)) = out.last_mut() {
                    let trimmed = prev.trim_end().len();
                    prev.truncate(trimmed);
                    if prev.is_empty() {
                        out.pop();
                    }
                }
                out.push(Inline::LineBreak);
            }
            other => out.push(other),
        }
    }
    out
}

/// Joins adjacent text nodes (e.g. a URL split by `<wbr>`), recursively.
fn merge_text(nodes: Vec<Inline>) -> Vec<Inline> {
    let mut out: Vec<Inline> = Vec::with_capacity(nodes.len());
    for node in nodes {
        let node = map_children(node, merge_text);
        if let (Some(Inline::Text(prev)), Inline::Text(next)) = (out.last_mut(), &node) {
            prev.push_str(next);
            continue;
        }
        out.push(node);
    }
    out
}

fn map_children(node: Inline, f: fn(Vec<Inline>) -> Vec<Inline>) -> Inline {
    match node {
        Inline::Link { url, children } => Inline::Link {
            url,
            children: f(children),
        },
        Inline::Greentext(c) => Inline::Greentext(f(c)),
        Inline::Spoiler(c) => Inline::Spoiler(f(c)),
        Inline::Bold(c) => Inline::Bold(f(c)),
        Inline::Italic(c) => Inline::Italic(f(c)),
        Inline::Underline(c) => Inline::Underline(f(c)),
        Inline::Strike(c) => Inline::Strike(f(c)),
        other => other,
    }
}

/// Turns bare `http(s)://` URLs in text into links (4chan, for one,
/// doesn't link them). Text already inside a link is left alone.
fn linkify(nodes: Vec<Inline>) -> Vec<Inline> {
    let mut out = Vec::with_capacity(nodes.len());
    for node in nodes {
        match node {
            Inline::Text(text) => split_urls(&text, &mut out),
            link @ Inline::Link { .. } => out.push(link),
            other => out.push(map_children(other, linkify)),
        }
    }
    out
}

fn split_urls(text: &str, out: &mut Vec<Inline>) {
    let mut rest = text;
    loop {
        let start = match (rest.find("https://"), rest.find("http://")) {
            (Some(a), Some(b)) => a.min(b),
            (Some(a), None) | (None, Some(a)) => a,
            (None, None) => break,
        };
        let tail = &rest[start..];
        let mut end = tail
            .find(|c: char| c.is_whitespace() || "<>\"'".contains(c))
            .unwrap_or(tail.len());
        // Sentence punctuation right after a URL isn't part of it.
        while end > 0 && tail[..end].ends_with(['.', ',', ')', ';', ':', '!', '?', ']']) {
            end -= 1;
        }
        let url = &tail[..end];
        if url.len() <= "https://".len() {
            // Just the scheme - not a link. Emit it as text and move on.
            let skip = start + url.len().max(1);
            out.push(Inline::Text(rest[..skip].to_owned()));
            rest = &rest[skip..];
            continue;
        }
        if start > 0 {
            out.push(Inline::Text(rest[..start].to_owned()));
        }
        out.push(Inline::Link {
            url: url.to_owned(),
            children: vec![Inline::Text(url.to_owned())],
        });
        rest = &tail[end..];
    }
    if !rest.is_empty() {
        out.push(Inline::Text(rest.to_owned()));
    }
}

// ---------------------------------------------------------------------
// Tree → text / markup
// ---------------------------------------------------------------------

fn push_plain(nodes: &[Inline], out: &mut String) {
    for node in nodes {
        match node {
            Inline::Text(t) | Inline::Code(t) | Inline::DeadQuote(t) => out.push_str(t),
            Inline::Quote { text, .. } => out.push_str(text),
            Inline::LineBreak => out.push('\n'),
            other => {
                if let Some(children) = children_of(other) {
                    push_plain(children, out);
                }
            }
        }
    }
}

fn push_pango(nodes: &[Inline], palette: &Palette, in_link: bool, out: &mut String) {
    let wrap = |open: &str, close: &str, children: &[Inline], in_link: bool, out: &mut String| {
        out.push_str(open);
        push_pango(children, palette, in_link, out);
        out.push_str(close);
    };
    for node in nodes {
        match node {
            Inline::Text(t) => out.push_str(&escape_markup(t)),
            Inline::LineBreak => out.push('\n'),
            Inline::Code(t) => {
                out.push_str("<tt>");
                out.push_str(&escape_markup(t));
                out.push_str("</tt>");
            }
            Inline::Quote { post_id, text, .. } => {
                if in_link {
                    out.push_str(&escape_markup(text));
                } else {
                    out.push_str(&format!(
                        "<a href=\"quote:{}\">{}</a>",
                        escape_markup(post_id),
                        escape_markup(text)
                    ));
                }
            }
            Inline::DeadQuote(t) => {
                out.push_str(&format!(
                    "<span foreground=\"{}\" strikethrough=\"true\">{}</span>",
                    palette.dead_quote,
                    escape_markup(t)
                ));
            }
            Inline::Link { url, children } => {
                if in_link {
                    push_pango(children, palette, true, out);
                } else {
                    wrap(
                        &format!("<a href=\"{}\">", escape_markup(url)),
                        "</a>",
                        children,
                        true,
                        out,
                    );
                }
            }
            Inline::Greentext(c) => wrap(
                &format!("<span foreground=\"{}\">", palette.greentext),
                "</span>",
                c,
                in_link,
                out,
            ),
            Inline::Spoiler(c) => wrap(
                &format!(
                    "<span background=\"{0}\" foreground=\"{0}\">",
                    palette.spoiler
                ),
                "</span>",
                c,
                in_link,
                out,
            ),
            Inline::Bold(c) => wrap("<b>", "</b>", c, in_link, out),
            Inline::Italic(c) => wrap("<i>", "</i>", c, in_link, out),
            Inline::Underline(c) => wrap("<u>", "</u>", c, in_link, out),
            Inline::Strike(c) => wrap("<s>", "</s>", c, in_link, out),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn palette() -> Palette {
        Palette {
            greentext: "#00ff00".into(),
            dead_quote: "#888888".into(),
            spoiler: "#000000".into(),
        }
    }

    fn text(s: &str) -> Inline {
        Inline::Text(s.into())
    }

    #[test]
    fn fourchan_comment() {
        let html = concat!(
            r##"<a href="#p123" class="quotelink">&gt;&gt;123</a><br>"##,
            r#"<span class="quote">&gt;be me</span><br>"#,
            r#"It&#039;s <s>secret</s> "#,
            r##"<a href="/g/thread/999#p1000" class="quotelink">&gt;&gt;1000</a><br>"##,
            r#"<span class="deadlink">&gt;&gt;555</span> see https://exa<wbr>mple.com/a."#,
        );
        let c = Comment::from_html(html);
        assert_eq!(
            c.nodes,
            vec![
                Inline::Quote {
                    post_id: "123".into(),
                    href: "#p123".into(),
                    text: ">>123".into()
                },
                Inline::LineBreak,
                Inline::Greentext(vec![text(">be me")]),
                Inline::LineBreak,
                text("It's "),
                Inline::Spoiler(vec![text("secret")]),
                text(" "),
                Inline::Quote {
                    post_id: "1000".into(),
                    href: "/g/thread/999#p1000".into(),
                    text: ">>1000".into()
                },
                Inline::LineBreak,
                Inline::DeadQuote(">>555".into()),
                text(" see "),
                Inline::Link {
                    url: "https://example.com/a".into(),
                    children: vec![text("https://example.com/a")]
                },
                text("."),
            ]
        );
        assert_eq!(c.quoted_ids(), vec!["123", "1000"]);
    }

    #[test]
    fn vichan_comment() {
        let html = r#"<a onclick="highlightReply('5', event);" href="/tech/res/1.html#5">&gt;&gt;5</a><br/><span class="spoiler">hidden</span>"#;
        let c = Comment::from_html(html);
        assert_eq!(c.quoted_ids(), vec!["5"]);
        assert_eq!(c.nodes[2], Inline::Spoiler(vec![text("hidden")]));
    }

    #[test]
    fn lynxchan_comment() {
        let html = r#"<a class="quoteLink" href="/b/res/10.html#11">&gt;&gt;11</a><br><span class="greenText">&gt;green</span><br><span class="redText">RED</span>"#;
        let c = Comment::from_html(html);
        assert_eq!(c.quoted_ids(), vec!["11"]);
        assert_eq!(c.nodes[2], Inline::Greentext(vec![text(">green")]));
        assert_eq!(c.nodes[4], Inline::Bold(vec![text("RED")]));
    }

    #[test]
    fn cross_board_link_is_a_link_not_a_quote() {
        let c = Comment::from_html(
            r#"<a href="//boards.4chan.org/g/" class="quotelink">&gt;&gt;&gt;/g/</a>"#,
        );
        assert!(c.quoted_ids().is_empty());
        assert!(matches!(c.nodes[0], Inline::Link { .. }));
    }

    #[test]
    fn whitespace_collapses_and_breaks_trim() {
        let c = Comment::from_html("one  two<br> three \n four <br>five");
        assert_eq!(c.plain_text(), "one two\nthree four\nfive");
    }

    #[test]
    fn scripts_dropped_and_paragraphs_break() {
        let c = Comment::from_html("<p>one</p><script>alert(1)</script><p>two</p>");
        assert_eq!(c.plain_text(), "one\ntwo");
    }

    #[test]
    fn plain_text_input() {
        let c = Comment::from_plain_text("line one\nline two http://x.org");
        assert_eq!(c.plain_text(), "line one\nline two http://x.org");
        assert!(matches!(c.nodes.last(), Some(Inline::Link { .. })));
    }

    #[test]
    fn excerpt_collapses_and_truncates() {
        let c = Comment::from_html("hello<br><br>   world and more text");
        assert_eq!(c.excerpt(100), "hello world and more text");
        assert_eq!(c.excerpt(9), "hello wo…");
    }

    #[test]
    fn pango_markup_is_escaped_and_styled() {
        let c = Comment::from_html(
            r##"<a href="#p1" class="quotelink">&gt;&gt;1</a> <span class="quote">&gt;a &lt;b&gt;</span> <s>x</s> <b>y</b>"##,
        );
        assert_eq!(
            c.to_pango(&palette()),
            concat!(
                r#"<a href="quote:1">&gt;&gt;1</a> "#,
                r##"<span foreground="#00ff00">&gt;a &lt;b&gt;</span> "##,
                r##"<span background="#000000" foreground="#000000">x</span> "##,
                "<b>y</b>"
            )
        );
    }

    #[test]
    fn link_href_is_escaped() {
        let c = Comment::from_html(r#"<a href="https://x.org/?a=1&amp;b=&quot;2&quot;">x</a>"#);
        assert_eq!(
            c.to_pango(&palette()),
            r#"<a href="https://x.org/?a=1&amp;b=&quot;2&quot;">x</a>"#
        );
    }

    #[test]
    fn quote_id_forms() {
        assert_eq!(quote_id("#p123").as_deref(), Some("123"));
        assert_eq!(quote_id("#q42").as_deref(), Some("42"));
        assert_eq!(quote_id("/b/res/1.html#2").as_deref(), Some("2"));
        assert_eq!(quote_id("/b/thread/77").as_deref(), Some("77"));
        assert_eq!(quote_id("/g/catalog#s=rust"), None);
    }
}
