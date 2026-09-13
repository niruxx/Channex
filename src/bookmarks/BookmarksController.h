#pragma once

#include <QObject>
#include <QQmlEngine>
#include <QVariantList>

// Mirrors src/store/useBookmarksStore.ts. Bookmark shape: { siteId,
// boardCode, threadId, subject, excerpt, thumbUrl, addedAt,
// lastSeenReplyCount }, keyed by "siteId/boardCode/threadId". Persisted
// under the "bookmarks" key of the same channex-qt-data.json file
// SettingsManager writes, via its own hydrate()/save() pair so the two
// controllers don't need to know about each other's fields.
class BookmarksController : public QObject
{
    Q_OBJECT
    QML_ELEMENT
    QML_SINGLETON

    Q_PROPERTY(QVariantList bookmarks READ bookmarks NOTIFY bookmarksChanged)

public:
    // Constructor stays private: a public one makes this look default-
    // constructible to QML_SINGLETON's factory detection, which can then
    // bypass create() and hand QML a disconnected instance.
    static BookmarksController *create(QQmlEngine *, QJSEngine *);
    static BookmarksController *instance();

    QVariantList bookmarks() const { return m_bookmarks; }

    Q_INVOKABLE void hydrate();
    Q_INVOKABLE bool isBookmarked(const QString &siteId, const QString &boardCode, const QString &threadId) const;
    Q_INVOKABLE void toggle(const QVariantMap &bookmark);
    Q_INVOKABLE void remove(const QString &siteId, const QString &boardCode, const QString &threadId);
    Q_INVOKABLE void markSeen(const QString &siteId, const QString &boardCode, const QString &threadId, int replyCount);

signals:
    void bookmarksChanged();

private:
    explicit BookmarksController(QObject *parent = nullptr);

    static QString keyOf(const QString &siteId, const QString &boardCode, const QString &threadId);
    int indexOf(const QString &siteId, const QString &boardCode, const QString &threadId) const;
    void save() const;

    QVariantList m_bookmarks;
};
