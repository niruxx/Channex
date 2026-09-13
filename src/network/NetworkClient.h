#pragma once

#include <QByteArray>
#include <QHttpMultiPart>
#include <QNetworkAccessManager>
#include <QObject>
#include <QString>
#include <functional>

// Thin async wrapper around QNetworkAccessManager. Every imageboard
// request (board list / catalog / thread / captcha / full-res media /
// reply submission / downloaded file bytes) goes through here, mirroring
// how the Tauri app funnels everything through a single HTTP plugin
// instance instead of ad-hoc fetch() calls.
class NetworkClient : public QObject
{
    Q_OBJECT

public:
    using BytesCallback = std::function<void(bool ok, const QByteArray &data, const QString &error)>;

    static NetworkClient *instance();

    void get(const QString &url, BytesCallback cb);
    void postMultipart(const QString &url, QHttpMultiPart *multiPart, BytesCallback cb);

private:
    explicit NetworkClient(QObject *parent = nullptr);
    QNetworkAccessManager m_manager;
};
