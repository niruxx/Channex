#pragma once

#include <QObject>
#include <QQmlEngine>
#include <QVariantMap>

class LightboxImageProvider;

// Registers the image provider that serves captcha PNGs at
// image://captcha/<key>; called once from main.cpp.
void setCaptchaImageProvider(LightboxImageProvider *provider);

// Mirrors src/lib/lynxchanPosting.ts: captcha fetch + multipart reply
// submission for postEngine:"lynxchan" sites (8kun, 8chan.moe).
class LynxchanPoster : public QObject
{
    Q_OBJECT
    QML_ELEMENT
    QML_SINGLETON

public:
    // Constructor stays private: a public one makes this look default-
    // constructible to QML_SINGLETON's factory detection, which can then
    // bypass create() and hand QML a disconnected instance.
    static LynxchanPoster *create(QQmlEngine *, QJSEngine *);

    // Emits captchaReady(imageProviderUrl) or captchaFailed(error).
    Q_INVOKABLE void fetchCaptcha(const QVariantMap &site, const QString &boardCode);

    // Emits postSucceeded(postId) or postFailed(error).
    Q_INVOKABLE void postReply(const QVariantMap &site, const QString &boardCode, const QString &threadId,
                                const QString &message, const QString &name, const QString &options,
                                const QString &captchaAnswer, bool spoiler);

    // Lazily generated once per app run and reused for every post,
    // matching sessionPassword() in lynxchanPosting.ts (LynxChan's
    // post-deletion-password convention).
    Q_INVOKABLE QString sessionPassword();

signals:
    void captchaReady(const QString &imageUrl);
    void captchaFailed(const QString &error);
    void postSucceeded(const QString &postId);
    void postFailed(const QString &error);

private:
    explicit LynxchanPoster(QObject *parent = nullptr);

    QString m_sessionPassword;
};
