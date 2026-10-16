#pragma once

#include <QByteArray>
#include <QHttpMultiPart>
#include <QNetworkAccessManager>
#include <QNetworkRequest>
#include <QObject>
#include <QQmlNetworkAccessManagerFactory>
#include <QString>
#include <functional>

// Thin async wrapper around QNetworkAccessManager. Every imageboard
// request (board list / catalog / thread / captcha / full-res media /
// reply submission / downloaded file bytes) goes through here, mirroring
// how the Tauri app funnels everything through a single HTTP plugin
// instance instead of ad-hoc fetch() calls.
//
// Requests are pinned to HTTP/1.1 with a transfer timeout - see
// configureRequest(). Thumbnails loaded directly by QML Image elements
// don't come through here, so QmlNetworkFactory applies the same
// settings to the QML engine's own network managers.
class NetworkClient : public QObject
{
    Q_OBJECT

public:
    using BytesCallback = std::function<void(bool ok, const QByteArray &data, const QString &error)>;

    static NetworkClient *instance();

    // Shared request policy: UA, redirects, HTTP/1.1, transfer timeout.
    static void configureRequest(QNetworkRequest &req);

    void get(const QString &url, BytesCallback cb);
    void postMultipart(const QString &url, QHttpMultiPart *multiPart, BytesCallback cb);

private:
    explicit NetworkClient(QObject *parent = nullptr);
    void getWithRetry(const QString &url, BytesCallback cb, int attemptsLeft);
    QNetworkAccessManager m_manager;
};

// Hands the QML engine network managers that apply
// NetworkClient::configureRequest() to every request they make
// (remote Image sources, i.e. catalog/post thumbnails and favicons).
class QmlNetworkFactory : public QQmlNetworkAccessManagerFactory
{
public:
    QNetworkAccessManager *create(QObject *parent) override;
};
