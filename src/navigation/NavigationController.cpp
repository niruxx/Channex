#include "NavigationController.h"

NavigationController *NavigationController::create(QQmlEngine *, QJSEngine *)
{
    return new NavigationController();
}

NavigationController::NavigationController(QObject *parent)
    : QObject(parent)
{
}

void NavigationController::push(const Frame &next)
{
    m_history.append(m_frame);
    m_frame = next;
    emit frameChanged();
}

void NavigationController::setSite(const QString &siteId)
{
    Frame f;
    f.view = QStringLiteral("catalog");
    f.siteId = siteId;
    f.boardCode = QString();
    push(f);
}

void NavigationController::goCatalog(const QString &siteId, const QString &boardCode)
{
    Frame f;
    f.view = QStringLiteral("catalog");
    f.siteId = siteId;
    f.boardCode = boardCode;
    push(f);
}

void NavigationController::goThread(const QString &siteId, const QString &boardCode, const QString &threadId)
{
    Frame f;
    f.view = QStringLiteral("thread");
    f.siteId = siteId;
    f.boardCode = boardCode;
    f.threadId = threadId;
    push(f);
}

void NavigationController::goBookmarks()
{
    Frame f = m_frame;
    f.view = QStringLiteral("bookmarks");
    push(f);
}

void NavigationController::goDownloads()
{
    Frame f = m_frame;
    f.view = QStringLiteral("downloads");
    push(f);
}

void NavigationController::goSettings()
{
    Frame f = m_frame;
    f.view = QStringLiteral("settings");
    push(f);
}

void NavigationController::back()
{
    if (m_history.isEmpty()) return;
    m_frame = m_history.takeLast();
    emit frameChanged();
}
