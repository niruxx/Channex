#include "NetworkClient.h"

#include <QNetworkReply>
#include <QNetworkRequest>
#include <QTimer>
#include <QUrl>

namespace {

// A stalled transfer errors out after this long instead of leaving the
// UI on a spinner forever. It's an inactivity timeout (reset whenever
// data arrives), so large media downloads that keep progressing are
// unaffected.
constexpr int kTransferTimeoutMs = 20000;

// Errors worth one silent retry: dropped/reset connections and
// timeouts, not 4xx/5xx answers from the server.
bool isTransient(QNetworkReply::NetworkError e)
{
    switch (e) {
    case QNetworkReply::RemoteHostClosedError:
    case QNetworkReply::TimeoutError:
    case QNetworkReply::OperationCanceledError:
    case QNetworkReply::TemporaryNetworkFailureError:
    case QNetworkReply::NetworkSessionFailedError:
    case QNetworkReply::UnknownNetworkError:
    case QNetworkReply::ProxyTimeoutError:
        return true;
    default:
        return false;
    }
}

// Applies configureRequest() to everything a QML engine loads.
class ConfiguredNetworkAccessManager : public QNetworkAccessManager
{
public:
    using QNetworkAccessManager::QNetworkAccessManager;

protected:
    QNetworkReply *createRequest(Operation op, const QNetworkRequest &original, QIODevice *data) override
    {
        QNetworkRequest req(original);
        const QString scheme = req.url().scheme();
        if (scheme == QLatin1String("http") || scheme == QLatin1String("https"))
            NetworkClient::configureRequest(req);
        return QNetworkAccessManager::createRequest(op, req, data);
    }
};

} // namespace

NetworkClient::NetworkClient(QObject *parent)
    : QObject(parent)
{
}

NetworkClient *NetworkClient::instance()
{
    static NetworkClient client;
    return &client;
}

void NetworkClient::configureRequest(QNetworkRequest &req)
{
    req.setHeader(QNetworkRequest::UserAgentHeader, QStringLiteral("Channex-Qt/0.1"));
    req.setAttribute(QNetworkRequest::RedirectPolicyAttribute, QNetworkRequest::NoLessSafeRedirectPolicy);
    // Qt negotiates HTTP/2 by default, and against the imageboard CDNs
    // (4cdn and friends) its HTTP/2 connection would regularly stall:
    // the first stream completes, later streams multiplexed onto the
    // same connection never finish - catalogs/threads stuck loading and
    // thumbnails never appearing. Plain HTTP/1.1 (Qt opens several
    // parallel connections per host) doesn't have that failure mode.
    req.setAttribute(QNetworkRequest::Http2AllowedAttribute, false);
    req.setTransferTimeout(kTransferTimeoutMs);
}

void NetworkClient::get(const QString &url, BytesCallback cb)
{
    getWithRetry(url, std::move(cb), 1);
}

void NetworkClient::getWithRetry(const QString &url, BytesCallback cb, int attemptsLeft)
{
    QNetworkRequest req{QUrl(url)};
    configureRequest(req);

    QNetworkReply *reply = m_manager.get(req);
    connect(reply, &QNetworkReply::finished, this, [this, reply, url, cb, attemptsLeft]() {
        reply->deleteLater();
        const auto error = reply->error();
        if (error != QNetworkReply::NoError) {
            if (attemptsLeft > 0 && isTransient(error)) {
                QTimer::singleShot(500, this, [this, url, cb, attemptsLeft]() {
                    getWithRetry(url, cb, attemptsLeft - 1);
                });
                return;
            }
            cb(false, {}, reply->errorString());
            return;
        }
        cb(true, reply->readAll(), {});
    });
}

void NetworkClient::postMultipart(const QString &url, QHttpMultiPart *multiPart, BytesCallback cb)
{
    // Not retried - re-sending a reply could double-post.
    QNetworkRequest req{QUrl(url)};
    configureRequest(req);

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

QNetworkAccessManager *QmlNetworkFactory::create(QObject *parent)
{
    return new ConfiguredNetworkAccessManager(parent);
}
