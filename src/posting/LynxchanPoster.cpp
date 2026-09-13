#include "LynxchanPoster.h"
#include "../lightbox/LightboxImageProvider.h"
#include "../network/NetworkClient.h"

#include <QHttpMultiPart>
#include <QHttpPart>
#include <QJsonDocument>
#include <QJsonObject>
#include <QJsonValue>
#include <QNetworkRequest>
#include <QRandomGenerator>

namespace {
LightboxImageProvider *g_captchaProvider = nullptr;
}

void setCaptchaImageProvider(LightboxImageProvider *provider)
{
    g_captchaProvider = provider;
}

LynxchanPoster *LynxchanPoster::create(QQmlEngine *, QJSEngine *)
{
    return new LynxchanPoster();
}

LynxchanPoster::LynxchanPoster(QObject *parent)
    : QObject(parent)
{
}

void LynxchanPoster::fetchCaptcha(const QVariantMap &site, const QString &boardCode)
{
    const qint64 bust = QRandomGenerator::global()->generate64();
    const QString url = QStringLiteral("%1/captcha.js?boardUri=%2&d=%3")
        .arg(site.value("apiOrigin").toString(), boardCode).arg(bust);

    NetworkClient::instance()->get(url, [this, url, bust](bool ok, const QByteArray &data, const QString &error) {
        if (!ok || !g_captchaProvider) {
            emit captchaFailed(error.isEmpty() ? QStringLiteral("failed to load captcha") : error);
            return;
        }
        const QString key = QStringLiteral("captcha-%1").arg(bust);
        g_captchaProvider->store(key, data);
        emit captchaReady(QStringLiteral("image://captcha/%1").arg(key));
    });
}

QString LynxchanPoster::sessionPassword()
{
    if (m_sessionPassword.isEmpty()) {
        const qint64 r = QRandomGenerator::global()->generate64();
        m_sessionPassword = QString::number(r, 36).right(8);
    }
    return m_sessionPassword;
}

void LynxchanPoster::postReply(const QVariantMap &site, const QString &boardCode, const QString &threadId,
                                const QString &message, const QString &name, const QString &options,
                                const QString &captchaAnswer, bool spoiler)
{
    const QString password = sessionPassword();
    auto *multiPart = new QHttpMultiPart(QHttpMultiPart::FormDataType);

    auto addField = [multiPart](const QString &name, const QString &value) {
        QHttpPart part;
        part.setHeader(QNetworkRequest::ContentDispositionHeader,
                       QVariant(QStringLiteral("form-data; name=\"%1\"").arg(name)));
        part.setBody(value.toUtf8());
        multiPart->append(part);
    };

    addField(QStringLiteral("boardUri"), boardCode);
    addField(QStringLiteral("threadId"), threadId);
    addField(QStringLiteral("message"), message);
    addField(QStringLiteral("name"), name);
    addField(QStringLiteral("email"), options);
    addField(QStringLiteral("captcha"), captchaAnswer);
    addField(QStringLiteral("password"), password);
    if (spoiler)
        addField(QStringLiteral("spoiler"), QStringLiteral("1"));

    const QString url = site.value("apiOrigin").toString() + QStringLiteral("/replyThread.js?json=1");

    NetworkClient::instance()->postMultipart(url, multiPart, [this](bool ok, const QByteArray &data, const QString &error) {
        const auto doc = QJsonDocument::fromJson(data);
        if (!doc.isObject()) {
            emit postFailed(ok ? QStringLiteral("site may be unavailable or the posting form changed") : error);
            return;
        }

        const auto obj = doc.object();
        const QString status = obj.value("status").toString();
        if (status != QStringLiteral("ok")) {
            const auto dataVal = obj.value("data");
            emit postFailed(dataVal.isString() ? dataVal.toString() : (status.isEmpty() ? error : status));
            return;
        }

        const auto dataVal = obj.value("data");
        emit postSucceeded(dataVal.isDouble() ? QString::number(dataVal.toInt()) : QStringLiteral("0"));
    });
}
