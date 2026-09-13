#pragma once

#include <QObject>
#include <QString>
#include <QVariantList>
#include <QVariantMap>
#include <functional>

// Direct C++ analogue of src/lib/adapters (ChanAdapter interface) and
// src/types.ts (ChanSite/Board/Post/PostFile/ThreadData) from the Tauri
// app. Posts/boards/files are passed around as QVariantMap so they can
// be handed to QML models without a separate value-type registration
// per field; the field *names* below are kept identical to types.ts so
// the two implementations stay comparable.
//
// Post fields: id, threadId, isOp, name, tripcode, capcode, subject,
//   timestamp, commentHtml, files (QVariantList<QVariantMap>), sticky,
//   closed, archived, countryCode, countryName, replyCount, imageCount,
//   omittedPosts, omittedImages
// PostFile fields: url, thumbUrl, name, ext, width, height, thumbWidth,
//   thumbHeight, size, isVideo, spoiler
// Board fields: code, title, description, nsfw

class ChanAdapter : public QObject
{
    Q_OBJECT
public:
    using BoardsCallback = std::function<void(QVariantList boards, QString error)>;
    using PostsCallback = std::function<void(QVariantList posts, QString error)>;
    using ThreadCallback = std::function<void(QVariantMap op, QVariantList replies, QString error)>;

    explicit ChanAdapter(QObject *parent = nullptr) : QObject(parent) {}
    ~ChanAdapter() override = default;

    virtual void fetchBoards(const QVariantMap &site, BoardsCallback cb) = 0;
    virtual void fetchCatalog(const QVariantMap &site, const QString &boardCode, PostsCallback cb) = 0;
    virtual void fetchThread(const QVariantMap &site, const QString &boardCode, const QString &threadId, ThreadCallback cb) = 0;
    virtual QString threadWebUrl(const QVariantMap &site, const QString &boardCode, const QString &threadId) const = 0;
};
