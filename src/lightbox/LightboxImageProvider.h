#pragma once

#include <QQuickImageProvider>
#include <QHash>
#include <QMutex>

// Backs image://lightbox/<key> requests. LightboxController fetches
// full-resolution media itself via NetworkClient (matching the Tauri
// app's documented workaround of not letting the webview's own <img>
// request full-res media directly) and stores the resulting bytes here
// keyed by post-file URL; Image elements in QML then load through this
// provider instead of pointing straight at the remote URL.
class LightboxImageProvider : public QQuickImageProvider
{
public:
    LightboxImageProvider();

    QImage requestImage(const QString &id, QSize *size, const QSize &requestedSize) override;

    void store(const QString &key, const QByteArray &bytes);

private:
    QMutex m_mutex;
    QHash<QString, QByteArray> m_cache;
};
