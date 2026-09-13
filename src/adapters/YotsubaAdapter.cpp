#include "YotsubaAdapter.h"
#include "CommentFormatter.h"
#include "../network/NetworkClient.h"

#include <QJsonArray>
#include <QJsonDocument>
#include <QJsonObject>
#include <QJsonValue>
#include <QSet>

namespace {

QString decodeEntities(QString s)
{
    s.replace(QStringLiteral("&gt;"), QStringLiteral(">"));
    s.replace(QStringLiteral("&lt;"), QStringLiteral("<"));
    s.replace(QStringLiteral("&quot;"), QStringLiteral("\""));
    s.replace(QStringLiteral("&#39;"), QStringLiteral("'"));
    s.replace(QStringLiteral("&amp;"), QStringLiteral("&"));
    return s;
}

struct MediaUrls { QString url; QString thumbUrl; };

MediaUrls mediaUrls(const QVariantMap &site, const QString &board, qint64 tim, const QString &ext)
{
    const QString mediaOrigin = site.value("mediaOrigin").toString();
    const QString layout = site.value("mediaLayout").toString();

    if (layout == QStringLiteral("flat-cdn")) {
        return { QStringLiteral("%1/%2/%3%4").arg(mediaOrigin, board).arg(tim).arg(ext),
                 QStringLiteral("%1/%2/%3s.jpg").arg(mediaOrigin, board).arg(tim) };
    }
    if (layout == QStringLiteral("file-store")) {
        return { QStringLiteral("%1/file_store/%2%3").arg(mediaOrigin).arg(tim).arg(ext),
                 QStringLiteral("%1/file_store/thumb/%2.jpg").arg(mediaOrigin).arg(tim) };
    }
    // board-dirs (vichan/Lainchan default)
    return { QStringLiteral("%1/%2/src/%3%4").arg(mediaOrigin, board).arg(tim).arg(ext),
             QStringLiteral("%1/%2/thumb/%3.jpg").arg(mediaOrigin, board).arg(tim) };
}

} // namespace

QVariantMap YotsubaAdapter::normalizePost(const QVariantMap &site, const QString &boardCode, const QVariantMap &raw)
{
    QVariantMap post;

    const qint64 no = raw.value("no").toLongLong();
    const qint64 resto = raw.value("resto").toLongLong();
    const bool isOp = resto == 0;

    post["id"] = QString::number(no);
    post["threadId"] = QString::number(isOp ? no : resto);
    post["isOp"] = isOp;
    post["name"] = decodeEntities(raw.value("name").toString());
    post["tripcode"] = raw.value("trip").toString();
    post["capcode"] = raw.value("capcode").toString();
    post["subject"] = decodeEntities(raw.value("sub").toString());
    post["timestamp"] = raw.value("time").toLongLong() * 1000;
    post["commentHtml"] = CommentFormatter::format(raw.value("com").toString());
    post["sticky"] = raw.value("sticky").toInt() != 0;
    post["closed"] = raw.value("closed").toInt() != 0;
    post["archived"] = raw.value("archived").toInt() != 0;
    post["countryCode"] = raw.value("country").toString();
    post["countryName"] = raw.value("country_name").toString();
    post["replyCount"] = raw.value("replies").toInt();
    post["imageCount"] = raw.value("images").toInt();
    post["omittedPosts"] = raw.value("omitted_posts").toInt();
    post["omittedImages"] = raw.value("omitted_images").toInt();

    QVariantList files;
    const bool filedeleted = raw.value("filedeleted").toInt() != 0;
    if (raw.contains("tim") && raw.contains("ext") && !filedeleted) {
        const qint64 tim = raw.value("tim").toLongLong();
        const QString ext = raw.value("ext").toString();
        const auto urls = mediaUrls(site, boardCode, tim, ext);

        static const QSet<QString> videoExts = { ".webm", ".mp4", ".mov" };

        QVariantMap file;
        file["url"] = urls.url;
        file["thumbUrl"] = urls.thumbUrl;
        file["name"] = decodeEntities(raw.value("filename").toString()) + ext;
        file["ext"] = ext;
        file["width"] = raw.value("w").toInt();
        file["height"] = raw.value("h").toInt();
        file["thumbWidth"] = raw.value("tn_w").toInt();
        file["thumbHeight"] = raw.value("tn_h").toInt();
        file["size"] = raw.value("fsize").toLongLong();
        file["isVideo"] = videoExts.contains(ext.toLower());
        file["spoiler"] = raw.value("spoiler").toInt() != 0;
        files.append(file);
    }
    post["files"] = files;

    return post;
}

void YotsubaAdapter::fetchBoards(const QVariantMap &site, BoardsCallback cb)
{
    const QString url = site.value("apiOrigin").toString() + QStringLiteral("/boards.json");
    const QVariantList fallback = site.value("defaultBoards").toList();

    NetworkClient::instance()->get(url, [fallback, cb](bool ok, const QByteArray &data, const QString &error) {
        if (!ok) { cb(fallback, error); return; }

        const auto doc = QJsonDocument::fromJson(data);
        QJsonArray arr;
        if (doc.isArray())
            arr = doc.array();
        else if (doc.isObject() && doc.object().contains("boards"))
            arr = doc.object().value("boards").toArray();
        else {
            cb(fallback, QStringLiteral("unexpected boards.json shape"));
            return;
        }

        QVariantList boards;
        for (const auto &v : arr) {
            const auto obj = v.toObject();
            QVariantMap board;
            board["code"] = obj.contains("uri") ? obj.value("uri").toString() : obj.value("board").toString();
            board["title"] = obj.contains("title") ? obj.value("title").toString() : obj.value("boardName").toString();
            board["description"] = obj.value("subtitle").toString(obj.value("meta_description").toString());
            boards.append(board);
        }

        if (boards.isEmpty()) { cb(fallback, {}); return; }
        cb(boards, {});
    });
}

void YotsubaAdapter::fetchCatalog(const QVariantMap &site, const QString &boardCode, PostsCallback cb)
{
    const QString url = site.value("apiOrigin").toString() + QStringLiteral("/%1/catalog.json").arg(boardCode);

    NetworkClient::instance()->get(url, [this, site, boardCode, cb](bool ok, const QByteArray &data, const QString &error) {
        if (!ok) { cb({}, error); return; }

        const auto doc = QJsonDocument::fromJson(data);
        if (!doc.isArray()) { cb({}, QStringLiteral("unexpected catalog.json shape")); return; }

        QVariantList posts;
        for (const auto &pageVal : doc.array()) {
            for (const auto &threadVal : pageVal.toObject().value("threads").toArray())
                posts.append(normalizePost(site, boardCode, threadVal.toObject().toVariantMap()));
        }
        cb(posts, {});
    });
}

void YotsubaAdapter::fetchThread(const QVariantMap &site, const QString &boardCode, const QString &threadId, ThreadCallback cb)
{
    const QString url = site.value("apiOrigin").toString() + QStringLiteral("/%1/thread/%2.json").arg(boardCode, threadId);

    NetworkClient::instance()->get(url, [this, site, boardCode, cb](bool ok, const QByteArray &data, const QString &error) {
        if (!ok) { cb({}, {}, error); return; }

        const auto doc = QJsonDocument::fromJson(data);
        const auto arr = doc.object().value("posts").toArray();
        if (arr.isEmpty()) { cb({}, {}, QStringLiteral("empty thread")); return; }

        QVariantMap op = normalizePost(site, boardCode, arr.first().toObject().toVariantMap());
        QVariantList replies;
        for (int i = 1; i < arr.size(); ++i)
            replies.append(normalizePost(site, boardCode, arr.at(i).toObject().toVariantMap()));

        cb(op, replies, {});
    });
}

QString YotsubaAdapter::threadWebUrl(const QVariantMap &site, const QString &boardCode, const QString &threadId) const
{
    const QString origin = site.value("siteOrigin").toString();
    if (site.value("mediaLayout").toString() == QStringLiteral("flat-cdn"))
        return QStringLiteral("%1/%2/thread/%3").arg(origin, boardCode, threadId);
    return QStringLiteral("%1/%2/res/%3.html").arg(origin, boardCode, threadId);
}
