#include "LightboxImageProvider.h"

#include <QMutexLocker>

LightboxImageProvider::LightboxImageProvider()
    : QQuickImageProvider(QQuickImageProvider::Image)
{
}

void LightboxImageProvider::store(const QString &key, const QByteArray &bytes)
{
    QMutexLocker locker(&m_mutex);
    m_cache.insert(key, bytes);
}

QImage LightboxImageProvider::requestImage(const QString &id, QSize *size, const QSize &requestedSize)
{
    // id arrives as "<key>?v=<cache-busting-counter>" - strip the query.
    const QString key = id.section('?', 0, 0);

    QByteArray bytes;
    {
        QMutexLocker locker(&m_mutex);
        bytes = m_cache.value(key);
    }

    QImage image;
    image.loadFromData(bytes);
    if (size) *size = image.size();
    if (requestedSize.isValid())
        return image.scaled(requestedSize, Qt::KeepAspectRatio, Qt::SmoothTransformation);
    return image;
}
