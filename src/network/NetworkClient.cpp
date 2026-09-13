#include "NetworkClient.h"

#include <QNetworkReply>
#include <QNetworkRequest>
#include <QUrl>

NetworkClient::NetworkClient(QObject *parent)
    : QObject(parent)
{
}

NetworkClient *NetworkClient::instance()
{
    static NetworkClient client;
    return &client;
}

void NetworkClient::get(const QString &url, BytesCallback cb)
{
    QNetworkRequest req{QUrl(url)};
    req.setHeader(QNetworkRequest::UserAgentHeader, QStringLiteral("Channex-Qt/0.1"));
    req.setAttribute(QNetworkRequest::RedirectPolicyAttribute, QNetworkRequest::NoLessSafeRedirectPolicy);

    QNetworkReply *reply = m_manager.get(req);
    connect(reply, &QNetworkReply::finished, this, [reply, cb]() {
        reply->deleteLater();
        if (reply->error() != QNetworkReply::NoError) {
            cb(false, {}, reply->errorString());
            return;
        }
        cb(true, reply->readAll(), {});
    });
}

void NetworkClient::postMultipart(const QString &url, QHttpMultiPart *multiPart, BytesCallback cb)
{
    QNetworkRequest req{QUrl(url)};
    req.setHeader(QNetworkRequest::UserAgentHeader, QStringLiteral("Channex-Qt/0.1"));

    QNetworkReply *reply = m_manager.post(req, multiPart);
    multiPart->setParent(reply);
    connect(reply, &QNetworkReply::finished, this, [reply, cb]() {
        reply->deleteLater();
        if (reply->error() != QNetworkReply::NoError) {
            cb(false, reply->readAll(), reply->errorString());
            return;
        }
        cb(true, reply->readAll(), {});
    });
}
