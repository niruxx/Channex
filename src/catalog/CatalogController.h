#pragma once

#include <QAbstractListModel>
#include <QQmlEngine>
#include <QVariantList>

// Mirrors src/hooks/useCatalog.ts + src/components/catalog/sort.ts:
// fetches a board's catalog through the current site's ChanAdapter and
// exposes it as a searchable, sortable list model for the QML catalog
// grid/list. Sticky threads always sort first regardless of sortMode,
// matching sortThreads() in the Tauri app.
class CatalogController : public QAbstractListModel
{
    Q_OBJECT
    QML_ELEMENT

    Q_PROPERTY(QString siteId READ siteId NOTIFY targetChanged)
    Q_PROPERTY(QString boardCode READ boardCode NOTIFY targetChanged)
    Q_PROPERTY(bool loading READ loading NOTIFY loadingChanged)
    Q_PROPERTY(QString errorString READ errorString NOTIFY errorChanged)
    Q_PROPERTY(QString searchQuery READ searchQuery WRITE setSearchQuery NOTIFY searchQueryChanged)
    Q_PROPERTY(QString sortMode READ sortMode WRITE setSortMode NOTIFY sortModeChanged)
    Q_PROPERTY(int count READ count NOTIFY dataReset)

public:
    enum Roles {
        IdRole = Qt::UserRole + 1,
        ThreadIdRole,
        SubjectRole,
        CommentHtmlRole,
        NameRole,
        TimestampRole,
        StickyRole,
        ClosedRole,
        ReplyCountRole,
        ImageCountRole,
        ThumbUrlRole,
        IsVideoRole,
        SpoilerRole,
        FileCountRole,
        PostRole,
    };
    Q_ENUM(Roles)

    explicit CatalogController(QObject *parent = nullptr);

    int rowCount(const QModelIndex &parent = QModelIndex()) const override;
    QVariant data(const QModelIndex &index, int role) const override;
    QHash<int, QByteArray> roleNames() const override;

    QString siteId() const { return m_siteId; }
    QString boardCode() const { return m_boardCode; }
    bool loading() const { return m_loading; }
    QString errorString() const { return m_error; }
    int count() const { return m_view.size(); }

    QString searchQuery() const { return m_searchQuery; }
    void setSearchQuery(const QString &q);

    QString sortMode() const { return m_sortMode; }
    void setSortMode(const QString &mode);

    Q_INVOKABLE void load(const QString &siteId, const QString &boardCode);
    Q_INVOKABLE void reload();

signals:
    void targetChanged();
    void loadingChanged();
    void errorChanged();
    void searchQueryChanged();
    void sortModeChanged();
    void dataReset();

private:
    void recomputeView();
    static QString plainTextExcerpt(const QVariantMap &post);

    QString m_siteId;
    QString m_boardCode;
    bool m_loading = false;
    QString m_error;
    QString m_searchQuery;
    QString m_sortMode = QStringLiteral("bump");

    QVariantList m_raw;   // as returned by the adapter
    QVariantList m_view;  // filtered + sorted for display
    int m_requestSeq = 0; // discards stale responses on rapid nav, like useCatalog's seq ref
};
