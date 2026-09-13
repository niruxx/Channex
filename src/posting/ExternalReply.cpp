#include "ExternalReply.h"

#include <QClipboard>
#include <QDesktopServices>
#include <QGuiApplication>
#include <QUrl>

ExternalReply *ExternalReply::create(QQmlEngine *, QJSEngine *)
{
    return new ExternalReply();
}

ExternalReply::ExternalReply(QObject *parent)
    : QObject(parent)
{
}

void ExternalReply::openForReply(const QString &threadUrl, const QString &commentToCopy)
{
    if (!commentToCopy.isEmpty())
        QGuiApplication::clipboard()->setText(commentToCopy);
    QDesktopServices::openUrl(QUrl(threadUrl));
}

void ExternalReply::openAuthPage(const QString &url)
{
    QDesktopServices::openUrl(QUrl(url));
}

void ExternalReply::openUrl(const QString &url)
{
    QDesktopServices::openUrl(QUrl(url));
}
