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
    Q_INVOKABLE void addCustomSite(const QVariantMap &input);
    Q_INVOKABLE void removeCustomSite(const QString &id);

signals:
    void sitesChanged();
    void currentSiteIdChanged();
    void boardsChanged();

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
