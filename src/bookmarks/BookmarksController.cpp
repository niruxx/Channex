#include "BookmarksController.h"

#include <QDateTime>
#include <QDir>
#include <QFile>
#include <QJsonArray>
#include <QJsonDocument>
#include <QJsonObject>
#include <QStandardPaths>

namespace { BookmarksController *g_instance = nullptr; }

BookmarksController *BookmarksController::create(QQmlEngine *, QJSEngine *)
{
    if (!g_instance)
        g_instance = new BookmarksController();
    return g_instance;
}

BookmarksController *BookmarksController::instance()
{
    if (!g_instance)
        g_instance = new BookmarksController();
    return g_instance;
}

BookmarksController::BookmarksController(QObject *parent)
    : QObject(parent)
{
}

QString BookmarksController::keyOf(const QString &siteId, const QString &boardCode, const QString &threadId)
{
    return siteId + '/' + boardCode + '/' + threadId;
}

int BookmarksController::indexOf(const QString &siteId, const QString &boardCode, const QString &threadId) const
{
    const QString key = keyOf(siteId, boardCode, threadId);
    for (int i = 0; i < m_bookmarks.size(); ++i) {
        const auto b = m_bookmarks.at(i).toMap();
        if (keyOf(b.value("siteId").toString(), b.value("boardCode").toString(), b.value("threadId").toString()) == key)
            return i;
    }
    return -1;
}

bool BookmarksController::isBookmarked(const QString &siteId, const QString &boardCode, const QString &threadId) const
{
    return indexOf(siteId, boardCode, threadId) >= 0;
}

void BookmarksController::toggle(const QVariantMap &bookmark)
{
    const QString siteId = bookmark.value("siteId").toString();
    const QString boardCode = bookmark.value("boardCode").toString();
    const QString threadId = bookmark.value("threadId").toString();

    const int idx = indexOf(siteId, boardCode, threadId);
    if (idx >= 0) {
        m_bookmarks.removeAt(idx);
    } else {
        QVariantMap b = bookmark;
        b["addedAt"] = QDateTime::currentMSecsSinceEpoch();
        b["lastSeenReplyCount"] = bookmark.value("replyCount", 0);
        m_bookmarks.append(b);
    }
    emit bookmarksChanged();
    save();
}

void BookmarksController::remove(const QString &siteId, const QString &boardCode, const QString &threadId)
{
    const int idx = indexOf(siteId, boardCode, threadId);
    if (idx < 0) return;
    m_bookmarks.removeAt(idx);
    emit bookmarksChanged();
    save();
}

void BookmarksController::markSeen(const QString &siteId, const QString &boardCode, const QString &threadId, int replyCount)
{
    const int idx = indexOf(siteId, boardCode, threadId);
    if (idx < 0) return;

    QVariantMap b = m_bookmarks.at(idx).toMap();
    if (b.value("lastSeenReplyCount").toInt() == replyCount) return;
    b["lastSeenReplyCount"] = replyCount;
    m_bookmarks[idx] = b;
    emit bookmarksChanged();
    save();
}

void BookmarksController::hydrate()
{
    const QString dir = QStandardPaths::writableLocation(QStandardPaths::AppDataLocation);
    QFile file(dir + QStringLiteral("/channex-qt-data.json"));
    if (!file.open(QIODevice::ReadOnly)) return;

    const auto doc = QJsonDocument::fromJson(file.readAll());
    m_bookmarks.clear();
    for (const auto &v : doc.object().value("bookmarks").toArray())
        m_bookmarks.append(v.toObject().toVariantMap());
    emit bookmarksChanged();
}

void BookmarksController::save() const
{
    const QString dirPath = QStandardPaths::writableLocation(QStandardPaths::AppDataLocation);
    QDir().mkpath(dirPath);
    const QString path = dirPath + QStringLiteral("/channex-qt-data.json");

    QJsonObject obj;
    QFile readFile(path);
    if (readFile.open(QIODevice::ReadOnly)) {
        obj = QJsonDocument::fromJson(readFile.readAll()).object();
        readFile.close();
    }

    QJsonArray arr;
    for (const auto &v : m_bookmarks)
        arr.append(QJsonObject::fromVariantMap(v.toMap()));
    obj["bookmarks"] = arr;

    QFile writeFile(path);
    if (writeFile.open(QIODevice::WriteOnly | QIODevice::Truncate))
        writeFile.write(QJsonDocument(obj).toJson(QJsonDocument::Indented));
}
