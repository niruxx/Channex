#include "LynxchanAdapter.h"
#include "CommentFormatter.h"
#include "../network/NetworkClient.h"

#include <QDateTime>
#include <QJsonArray>
#include <QJsonDocument>
#include <QJsonObject>

QString LynxchanAdapter::resolveUrl(const QVariantMap &site, const QString &path)
{
    if (path.startsWith(QStringLiteral("http://")) || path.startsWith(QStringLiteral("https://")))
        return path;
    return site.value("mediaOrigin").toString() + path;
}

QVariantMap LynxchanAdapter::fileOf(const QVariantMap &site, const QVariantMap &rawFile)
{
    QVariantMap file;
    const QString path = rawFile.value("path").toString();
    const QString thumb = rawFile.value("thumb").toString();
    const QString mime = rawFile.value("mime").toString();

    file["url"] = resolveUrl(site, path);
    file["thumbUrl"] = thumb.isEmpty() ? file["url"] : resolveUrl(site, thumb);
    file["name"] = rawFile.value("originalName").toString();
    file["ext"] = QStringLiteral(".") + path.section('.', -1);
    file["width"] = rawFile.value("width");
    file["height"] = rawFile.value("height");
    file["thumbWidth"] = rawFile.value("width");
    file["thumbHeight"] = rawFile.value("height");
    file["size"] = rawFile.value("size");
    file["isVideo"] = mime.startsWith(QStringLiteral("video/"));
    file["spoiler"] = false;
    return file;
}

QString LynxchanAdapter::commentHtmlOf(const QVariantMap &raw)
{
    const QString markdown = raw.value("markdown").toString();
    if (!markdown.isEmpty())
        return CommentFormatter::format(markdown);

    const QString message = raw.value("message").toString();
    return CommentFormatter::format(QStringLiteral("<p>%1</p>").arg(message.toHtmlEscaped()));
}

void LynxchanAdapter::fetchBoards(const QVariantMap &site, BoardsCallback cb)
{
    const QString siteOrigin = site.value("siteOrigin").toString();
    const QVariantList fallback = site.value("defaultBoards").toList();

    auto parseBoardArray = [](const QJsonArray &arr) {
        QVariantList boards;
        for (const auto &v : arr) {
            const auto obj = v.toObject();
            QVariantMap board;
            board["code"] = obj.contains("uri") ? obj.value("uri").toString() : obj.value("boardUri").toString();
            board["title"] = obj.contains("title") ? obj.value("title").toString() : obj.value("boardName").toString();
            board["description"] = obj.value("subtitle").toString();
            boards.append(board);
        }
        return boards;
    };

    NetworkClient::instance()->get(siteOrigin + QStringLiteral("/boards.json"),
        [siteOrigin, fallback, parseBoardArray, cb](bool ok, const QByteArray &data, const QString &) {
            if (ok) {
                const auto doc = QJsonDocument::fromJson(data);
                if (doc.isArray()) {
                    const auto boards = parseBoardArray(doc.array());
                    if (!boards.isEmpty()) { cb(boards, {}); return; }
                }
            }

            // Fall back to index.json's topBoards[]
            NetworkClient::instance()->get(siteOrigin + QStringLiteral("/index.json"),
                [fallback, parseBoardArray, cb](bool ok2, const QByteArray &data2, const QString &error2) {
                    if (ok2) {
                        const auto doc2 = QJsonDocument::fromJson(data2);
                        const auto boards = parseBoardArray(doc2.object().value("topBoards").toArray());
                        if (!boards.isEmpty()) { cb(boards, {}); return; }
                    }
                    cb(fallback, error2);
                });
        });
}

void LynxchanAdapter::fetchCatalog(const QVariantMap &site, const QString &boardCode, PostsCallback cb)
{
    const QString url = site.value("apiOrigin").toString() + QStringLiteral("/%1/catalog.json").arg(boardCode);

    NetworkClient::instance()->get(url, [site, boardCode, cb](bool ok, const QByteArray &data, const QString &error) {
        if (!ok) { cb({}, error); return; }

        const auto doc = QJsonDocument::fromJson(data);
        if (!doc.isArray()) { cb({}, QStringLiteral("unexpected catalog.json shape")); return; }

        QVariantList posts;
        for (const auto &v : doc.array()) {
            const auto raw = v.toObject().toVariantMap();
            QVariantMap post;

            const QString threadId = raw.value("threadId").toString();
            post["id"] = threadId;
            post["threadId"] = threadId;
            post["isOp"] = true;
            post["name"] = QString();
            post["tripcode"] = QString();
            post["capcode"] = QString();
            post["subject"] = raw.value("subject").toString();
            post["commentHtml"] = commentHtmlOf(raw);
            post["sticky"] = raw.value("pinned").toBool();
            post["closed"] = raw.value("locked").toBool();
            post["archived"] = false;
            post["replyCount"] = qMax(0, raw.value("postCount").toInt() - 1);
            post["imageCount"] = raw.value("fileCount").toInt();

            const QString lastBump = raw.value("lastBump").toString();
            const auto dt = QDateTime::fromString(lastBump, Qt::ISODate);
            post["timestamp"] = dt.isValid() ? dt.toMSecsSinceEpoch() : QDateTime::currentMSecsSinceEpoch();

            QVariantList files;
            const QString thumb = raw.value("thumb").toString();
            if (!thumb.isEmpty()) {
                QVariantMap rawFile;
                rawFile["path"] = thumb;
                rawFile["thumb"] = thumb;
                rawFile["mime"] = raw.value("mime");
                rawFile["originalName"] = QString();
                files.append(fileOf(site, rawFile));
            }
            post["files"] = files;

            posts.append(post);
        }
        cb(posts, {});
    });
}

void LynxchanAdapter::fetchThread(const QVariantMap &site, const QString &boardCode, const QString &threadId, ThreadCallback cb)
{
    const QString url = site.value("apiOrigin").toString() + QStringLiteral("/%1/res/%2.json").arg(boardCode, threadId);

    NetworkClient::instance()->get(url, [site, threadId, cb](bool ok, const QByteArray &data, const QString &error) {
        if (!ok) { cb({}, {}, error); return; }

        const auto raw = QJsonDocument::fromJson(data).object().toVariantMap();

        auto buildPost = [&](const QVariantMap &p, bool isOp) {
            QVariantMap post;
            const QString postId = p.contains("postId") ? p.value("postId").toString() : threadId;
            post["id"] = postId;
            post["threadId"] = threadId;
            post["isOp"] = isOp;
            post["name"] = p.value("name").toString();
            post["tripcode"] = QString();
            post["capcode"] = p.value("signedRole").toString();
            post["subject"] = isOp ? raw.value("subject").toString() : QString();
            post["commentHtml"] = commentHtmlOf(p);
            post["sticky"] = isOp ? raw.value("pinned").toBool() : false;
            post["closed"] = isOp ? raw.value("locked").toBool() : false;
            post["archived"] = isOp ? raw.value("archived").toBool() : false;

            const QString creation = p.value("creation").toString();
            const auto dt = QDateTime::fromString(creation, Qt::ISODate);
            post["timestamp"] = dt.isValid() ? dt.toMSecsSinceEpoch() : QDateTime::currentMSecsSinceEpoch();

            QVariantList files;
            for (const auto &f : p.value("files").toList())
                files.append(fileOf(site, f.toMap()));
            post["files"] = files;

            return post;
        };

        QVariantMap op = buildPost(raw, true);
        QVariantList replies;
        for (const auto &p : raw.value("posts").toList())
            replies.append(buildPost(p.toMap(), false));

        cb(op, replies, {});
    });
}

QString LynxchanAdapter::threadWebUrl(const QVariantMap &site, const QString &boardCode, const QString &threadId) const
{
    return QStringLiteral("%1/%2/res/%3.html").arg(site.value("siteOrigin").toString(), boardCode, threadId);
}
