#pragma once

#include <QAbstractListModel>
#include <QMap>
#include <QQmlEngine>
#include <QVariantList>
#include <QVariantMap>

// Mirrors src/hooks/useThread.ts + the backlink-scanning logic in
// ThreadView.tsx: fetches OP + replies through the site's ChanAdapter,
// exposes them as one list model (row 0 = OP), and precomputes a
// postId -> [replying post ids] backlink map by scanning each post's
// commentHtml for quote:<id> links (see CommentFormatter).
class ThreadController : public QAbstractListModel
{
    Q_OBJECT
    QML_ELEMENT

    Q_PROPERTY(QString siteId READ siteId NOTIFY targetChanged)
    Q_PROPERTY(QString boardCode READ boardCode NOTIFY targetChanged)
    Q_PROPERTY(QString threadId READ threadId NOTIFY targetChanged)
    Q_PROPERTY(bool loading READ loading NOTIFY loadingChanged)
    Q_PROPERTY(QString errorString READ errorString NOTIFY errorChanged)
    Q_PROPERTY(QString subject READ subject NOTIFY dataReset)
    Q_PROPERTY(int postCount READ postCount NOTIFY dataReset)
    Q_PROPERTY(QString threadWebUrl READ threadWebUrl NOTIFY dataReset)
    Q_PROPERTY(QVariantList allFiles READ allFiles NOTIFY dataReset)

public:
    enum Roles {
        IdRole = Qt::UserRole + 1,
        IsOpRole,
        NameRole,
        TripcodeRole,
        CapcodeRole,
        SubjectRole,
        TimestampRole,
        CommentHtmlRole,
        FilesRole,
        StickyRole,
        ClosedRole,
        CountryCodeRole,
        CountryNameRole,
        BacklinksRole,
        PostRole,
    };
    Q_ENUM(Roles)

    explicit ThreadController(QObject *parent = nullptr);

    int rowCount(const QModelIndex &parent = QModelIndex()) const override;
    QVariant data(const QModelIndex &index, int role) const override;
    QHash<int, QByteArray> roleNames() const override;

    QString siteId() const { return m_siteId; }
    QString boardCode() const { return m_boardCode; }
    QString threadId() const { return m_threadId; }
    bool loading() const { return m_loading; }
    QString errorString() const { return m_error; }
    QString subject() const;
    int postCount() const { return m_posts.size(); }
    QString threadWebUrl() const { return m_threadWebUrl; }
    QVariantList allFiles() const;

    Q_INVOKABLE void load(const QString &siteId, const QString &boardCode, const QString &threadId);
    Q_INVOKABLE void reload();
    Q_INVOKABLE QStringList backlinksFor(const QString &postId) const;
    Q_INVOKABLE int rowForPostId(const QString &postId) const;

signals:
    void targetChanged();
    void loadingChanged();
    void errorChanged();
    void dataReset();
    void loaded(int replyCount); // used by QML to call BookmarksController::markSeen

private:
    void computeBacklinks();

    QString m_siteId, m_boardCode, m_threadId;
    bool m_loading = false;
    QString m_error;
    QString m_threadWebUrl;
    QVariantList m_posts; // [0] = OP, rest = replies
    QMap<QString, QStringList> m_backlinks;
    int m_requestSeq = 0;
};
