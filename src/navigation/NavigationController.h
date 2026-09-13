#pragma once

#include <QObject>
#include <QQmlEngine>
#include <QString>
#include <QVariantList>
#include <QVariantMap>
#include <QVector>

// Mirrors src/store/useNavStore.ts: a hand-rolled router (no page
// stack widget) with a linear history list. view is one of "catalog",
// "thread", "bookmarks", "downloads", "settings".
class NavigationController : public QObject
{
    Q_OBJECT
    QML_ELEMENT
    QML_SINGLETON

    Q_PROPERTY(QString view READ view NOTIFY frameChanged)
    Q_PROPERTY(QString siteId READ siteId NOTIFY frameChanged)
    Q_PROPERTY(QString boardCode READ boardCode NOTIFY frameChanged)
    Q_PROPERTY(QString threadId READ threadId NOTIFY frameChanged)
    Q_PROPERTY(bool canGoBack READ canGoBack NOTIFY frameChanged)

public:
    // Constructor stays private: a public one makes this look default-
    // constructible to QML_SINGLETON's factory detection, which can then
    // bypass create() and hand QML a disconnected instance.
    static NavigationController *create(QQmlEngine *, QJSEngine *);

    QString view() const { return m_frame.view; }
    QString siteId() const { return m_frame.siteId; }
    QString boardCode() const { return m_frame.boardCode; }
    QString threadId() const { return m_frame.threadId; }
    bool canGoBack() const { return !m_history.isEmpty(); }

    Q_INVOKABLE void setSite(const QString &siteId);
    Q_INVOKABLE void goCatalog(const QString &siteId, const QString &boardCode);
    Q_INVOKABLE void goThread(const QString &siteId, const QString &boardCode, const QString &threadId);
    Q_INVOKABLE void goBookmarks();
    Q_INVOKABLE void goDownloads();
    Q_INVOKABLE void goSettings();
    Q_INVOKABLE void back();

signals:
    void frameChanged();

private:
    explicit NavigationController(QObject *parent = nullptr);

    struct Frame {
        QString view = QStringLiteral("catalog");
        QString siteId = QStringLiteral("4chan");
        QString boardCode = QStringLiteral("g");
        QString threadId;
    };

    void push(const Frame &next);

    Frame m_frame;
    QVector<Frame> m_history;
};
