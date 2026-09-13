#include "CommentFormatter.h"

#include <QRegularExpression>
#include <QSet>
#include <QStringList>

namespace {

const QSet<QString> &allowedTags()
{
    static const QSet<QString> tags = {
        QStringLiteral("a"), QStringLiteral("b"), QStringLiteral("strong"),
        QStringLiteral("i"), QStringLiteral("em"), QStringLiteral("u"),
        QStringLiteral("s"), QStringLiteral("del"), QStringLiteral("span"),
        QStringLiteral("br"), QStringLiteral("p"), QStringLiteral("pre"),
        QStringLiteral("code"), QStringLiteral("blockquote"), QStringLiteral("sub"),
        QStringLiteral("sup"), QStringLiteral("small"), QStringLiteral("wbr"),
        QStringLiteral("ul"), QStringLiteral("ol"), QStringLiteral("li"),
    };
    return tags;
}

QString attrValue(const QString &tag, const QString &name)
{
    const QRegularExpression re(name + QStringLiteral("\\s*=\\s*\"([^\"]*)\""), QRegularExpression::CaseInsensitiveOption);
    const auto m = re.match(tag);
    return m.hasMatch() ? m.captured(1) : QString();
}

// Removes any <tag>/</tag> whose name is not in the allow-list, keeping
// the inner text content intact (equivalent to DOMPurify's default tag
// stripping, minus attribute filtering which happens in later passes).
QString stripDisallowedTags(const QString &html)
{
    QString out;
    out.reserve(html.size());

    static const QRegularExpression tagRe(QStringLiteral("<(/?)\\s*([a-zA-Z0-9]+)([^>]*)>"));
    int last = 0;
    auto it = tagRe.globalMatch(html);
    while (it.hasNext()) {
        const auto m = it.next();
        out += html.mid(last, m.capturedStart() - last);
        last = m.capturedEnd();

        const QString name = m.captured(2).toLower();
        if (allowedTags().contains(name))
            out += m.captured(0);
        // else: drop the tag marker, content around it is kept via 'last' bookkeeping
    }
    out += html.mid(last);
    return out;
}

// Rewrites <a ...> opening tags: quote/backlink anchors become
// href="quote:<id>", everything else is left as a normal external link.
QString rewriteAnchors(const QString &html)
{
    QString out;
    out.reserve(html.size());

    static const QRegularExpression aRe(QStringLiteral("<a\\b([^>]*)>"), QRegularExpression::CaseInsensitiveOption);
    int last = 0;
    auto it = aRe.globalMatch(html);
    while (it.hasNext()) {
        const auto m = it.next();
        out += html.mid(last, m.capturedStart() - last);
        last = m.capturedEnd();

        const QString attrs = m.captured(1);
        const QString cls = attrValue(attrs, QStringLiteral("class"));
        const QString href = attrValue(attrs, QStringLiteral("href"));

        const bool classLooksLikeQuote =
            cls.contains(QStringLiteral("quotelink"), Qt::CaseInsensitive) ||
            (cls.contains(QStringLiteral("quote"), Qt::CaseInsensitive) &&
             !cls.trimmed().endsWith(QStringLiteral("quote"), Qt::CaseInsensitive));
        const bool hrefLooksLikeQuote = href.startsWith(QStringLiteral("#p"), Qt::CaseInsensitive);

        QString quoteId;
        if (classLooksLikeQuote || hrefLooksLikeQuote) {
            static const QRegularExpression idRe1(QStringLiteral("#p?(\\d+)"));
            static const QRegularExpression idRe2(QStringLiteral("/(\\d+)$"));
            auto idm = idRe1.match(href);
            if (!idm.hasMatch())
                idm = idRe2.match(href);
            if (idm.hasMatch())
                quoteId = idm.captured(1);
        }

        if (!quoteId.isEmpty()) {
            out += QStringLiteral("<a href=\"quote:%1\" style=\"color:#7fb0e0;text-decoration:none;\">").arg(quoteId);
        } else if (cls.contains(QStringLiteral("deadlink"), Qt::CaseInsensitive)) {
            out += QStringLiteral("<a href=\"%1\" style=\"color:#8a8f98;text-decoration:line-through;\">").arg(href.toHtmlEscaped());
        } else {
            out += QStringLiteral("<a href=\"%1\" style=\"color:#7fb0e0;text-decoration:underline;\">").arg(href.toHtmlEscaped());
        }
    }
    out += html.mid(last);
    return out;
}

// Turns known imageboard <span class="..."> markers into inline styles
// since QML's Text.RichText has no stylesheet/class support.
QString rewriteSpans(const QString &html)
{
    QString out;
    out.reserve(html.size());

    static const QRegularExpression spanRe(QStringLiteral("<span\\b([^>]*)>"), QRegularExpression::CaseInsensitiveOption);
    int last = 0;
    auto it = spanRe.globalMatch(html);
    while (it.hasNext()) {
        const auto m = it.next();
        out += html.mid(last, m.capturedStart() - last);
        last = m.capturedEnd();

        const QString cls = attrValue(m.captured(1), QStringLiteral("class"));
        if (cls.contains(QStringLiteral("quote"), Qt::CaseInsensitive) && !cls.contains(QStringLiteral("quotelink"), Qt::CaseInsensitive))
            out += QStringLiteral("<span style=\"color:#86b661;\">");
        else if (cls.contains(QStringLiteral("spoiler"), Qt::CaseInsensitive))
            // Static rich text can't do the real hover-to-reveal; this
            // is a documented simplification vs. the Tauri version.
            out += QStringLiteral("<span style=\"background-color:#1a1a1a;color:#1a1a1a;\">");
        else
            out += QStringLiteral("<span>");
    }
    out += html.mid(last);
    return out;
}

// Strips leftover attributes from every other allowed tag (b, i, p, ...).
QString stripPlainTagAttributes(QString html)
{
    static const QStringList plainTags = {
        "b", "strong", "i", "em", "u", "s", "del", "p", "pre", "code",
        "blockquote", "sub", "sup", "small", "ul", "ol", "li",
    };
    for (const auto &tag : plainTags) {
        const QRegularExpression re(QStringLiteral("<%1\\b[^>]*>").arg(tag), QRegularExpression::CaseInsensitiveOption);
        html.replace(re, QStringLiteral("<%1>").arg(tag));
    }
    html.replace(QRegularExpression(QStringLiteral("<br\\b[^>]*>"), QRegularExpression::CaseInsensitiveOption),
                 QStringLiteral("<br/>"));
    html.replace(QRegularExpression(QStringLiteral("<wbr\\b[^>]*>"), QRegularExpression::CaseInsensitiveOption),
                 QStringLiteral("<wbr/>"));
    return html;
}

} // namespace

QString CommentFormatter::format(const QString &rawHtml)
{
    QString html = rawHtml;

    static const QRegularExpression scriptOrStyle(
        QStringLiteral("<(script|style)\\b[^>]*>[\\s\\S]*?</\\1>"),
        QRegularExpression::CaseInsensitiveOption);
    html.remove(scriptOrStyle);

    html = stripDisallowedTags(html);
    html = rewriteAnchors(html);
    html = rewriteSpans(html);
    html = stripPlainTagAttributes(html);

    return html;
}
