#pragma once

#include <QObject>
#include <QQmlEngine>
#include <QVariantList>
#include <QVariantMap>
#include <QMap>

// Mirrors src/store/useSitesStore.ts + src/lib/sites.ts: the 4 preset
// sites plus any user-added custom sites, and a per-site board
// cache/loading/error map populated by ChanAdapter::fetchBoards().
class SitesController : public QObject
{
    Q_OBJECT
    QML_ELEMENT
    QML_SINGLETON

    Q_PROPERTY(QVariantList sites READ sites NOTIFY sitesChanged)
    Q_PROPERTY(QString currentSiteId READ currentSiteId WRITE setCurrentSiteId NOTIFY currentSiteIdChanged)
    Q_PROPERTY(QVariantMap currentSite READ currentSite NOTIFY currentSiteIdChanged)
    Q_PROPERTY(QVariantList currentBoards READ currentBoards NOTIFY boardsChanged)
    Q_PROPERTY(bool boardsLoading READ boardsLoading NOTIFY boardsChanged)
    Q_PROPERTY(QString boardsError READ boardsError NOTIFY boardsChanged)

public:
    // Constructor stays private: a public one makes this look default-
    // constructible to QML_SINGLETON's factory detection, which can then
    // bypass create() and hand QML a disconnected instance.
    static SitesController *create(QQmlEngine *, QJSEngine *);
    static SitesController *instance();

    QVariantList sites() const;
    QString currentSiteId() const { return m_currentSiteId; }
    void setCurrentSiteId(const QString &id);
    QVariantMap currentSite() const { return findSite(m_currentSiteId); }

    QVariantList currentBoards() const;
    bool boardsLoading() const;
    QString boardsError() const;

    Q_INVOKABLE QVariantMap findSite(const QString &id) const;
    Q_INVOKABLE void loadBoards(const QString &siteId);

    // Boards for an arbitrary site, independent of currentSiteId/
    // currentBoards (which only ever track the app's active site) - for
    // Settings' board-visibility manager, which lets a user browse a
    // site's boards without switching the sidebar/catalog to it. Kicks
    // off a background fetch to warm the cache when needed, but returns
    // synchronously: the sorted default board list as an immediate
    // fallback, or the cached/fetched list once one exists.
    Q_INVOKABLE QVariantList boardsForSite(const QString &siteId);
    Q_INVOKABLE void addCustomSite(const QVariantMap &input);
    Q_INVOKABLE void removeCustomSite(const QString &id);

    // Parses a thread URL (4chan-style "/<board>/thread/<id>" or
    // vichan/LynxChan-style "/<board>/res/<id>.html") and matches its
    // host against an already-registered site's siteOrigin. Returns
    // {siteId, boardCode, threadId} on success, or an empty map if the
    // text isn't a recognizable thread URL or its host isn't a known
    // site - used to let pasting a thread link into the catalog search
    // field jump straight to that thread (see CatalogToolbar.qml).
    Q_INVOKABLE QVariantMap resolveThreadUrl(const QString &url) const;

signals:
    void sitesChanged();
    void currentSiteIdChanged();
    void boardsChanged();
    // Fires for every loadBoards() completion, regardless of siteId -
    // unlike boardsChanged() (which only fires for currentSiteId), this
    // is what lets boardsForSite()'s consumers (Settings' board manager)
    // know a background fetch for a NON-active site just finished, so
    // they can move off the synchronous defaultBoards fallback.
    void boardsForSiteChanged(const QString &siteId);

private:
    explicit SitesController(QObject *parent = nullptr);

    struct BoardState {
        QVariantList boards;
        bool loading = false;
        QString error;
    };

    QVariantList presetSites() const;
    void mergeDefaultBoards(const QString &siteId, QVariantList &boards) const;

    QString m_currentSiteId = QStringLiteral("4chan");
    QMap<QString, BoardState> m_boardState;
};
