#pragma once

#include <QString>

// Lightweight stand-in for src/lib/sanitize.ts (DOMPurify + custom
// afterSanitizeAttributes hook). Strips comment HTML down to a small
// allow-list of tags, then rewrites <a> tags so QML's Text.RichText
// (backed by QTextDocument) can drive navigation via its native
// onLinkActivated signal instead of DOM data-attributes:
//   - a quote/backlink (class contains "quote"/"quotelink", or href
//     looks like "#p12345") becomes <a href="quote:12345">
//   - anything else becomes a normal external link, left as-is
// This is a regex-based approximation, not a real HTML parser -
// sufficient for the simple, mostly-flat markup imageboard engines
// emit, but not a hardened sanitizer. Good enough for a placeholder
// build; revisit with a real parser before this replaces the Tauri app.
namespace CommentFormatter {

QString format(const QString &rawHtml);

}
