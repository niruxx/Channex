#include "LightboxController.h"
#include "LightboxImageProvider.h"
#include "../network/NetworkClient.h"

namespace {
LightboxController *g_instance = nullptr;
LightboxImageProvider *g_provider = nullptr;
}

LightboxController *LightboxController::create(QQmlEngine *, QJSEngine *)
{
    if (!g_instance)
        g_instance = new LightboxController();
    return g_instance;
}

LightboxController *LightboxController::instance()
{
    if (!g_instance)
        g_instance = new LightboxController();
    return g_instance;
}

void LightboxController::setImageProvider(LightboxImageProvider *provider)
{
    g_provider = provider;
}

LightboxController::LightboxController(QObject *parent)
    : QObject(parent)
{
}

QVariantMap LightboxController::currentFile() const
{
    if (m_index < 0 || m_index >= m_files.size()) return {};
    return m_files.at(m_index).toMap();
}

QString LightboxController::imageSource() const
{
    if (m_loadState != QStringLiteral("loaded")) return {};
    const auto file = currentFile();
    if (file.value("isVideo").toBool()) return {};
    return QStringLiteral("image://lightbox/%1?v=%2").arg(file.value("url").toString()).arg(m_cacheBust);
}

void LightboxController::setZoomed(bool z)
{
    if (m_zoomed == z) return;
    m_zoomed = z;
    emit zoomedChanged();
}

void LightboxController::open(const QVariantList &files, int startIndex)
{
    m_files = files;
    m_index = qBound(0, startIndex, qMax(0, files.size() - 1));
    m_isOpen = true;
    m_zoomed = false;
    emit filesChanged();
    emit indexChanged();
    emit isOpenChanged();
    emit zoomedChanged();
    loadCurrent();
}

void LightboxController::close()
{
    if (!m_isOpen) return;
    m_isOpen = false;
    emit isOpenChanged();
}

void LightboxController::next()
{
    if (m_files.isEmpty()) return;
    m_index = (m_index + 1) % m_files.size();
    m_zoomed = false;
    emit indexChanged();
    emit zoomedChanged();
    loadCurrent();
}

void LightboxController::prev()
{
    if (m_files.isEmpty()) return;
    m_index = (m_index - 1 + m_files.size()) % m_files.size();
    m_zoomed = false;
    emit indexChanged();
    emit zoomedChanged();
    loadCurrent();
}

void LightboxController::loadCurrent()
{
    const auto file = currentFile();
    if (file.isEmpty()) return;

    if (file.value("isVideo").toBool()) {
        m_loadState = QStringLiteral("loaded");
        emit loadStateChanged();
        return;
    }

    m_loadState = QStringLiteral("loading");
    emit loadStateChanged();

    const int seq = ++m_requestSeq;
    const QString url = file.value("url").toString();

    NetworkClient::instance()->get(url, [this, seq, url](bool ok, const QByteArray &data, const QString &) {
        if (seq != m_requestSeq) return;

        if (!ok || !g_provider) {
            m_loadState = QStringLiteral("error");
            emit loadStateChanged();
            return;
        }

        g_provider->store(url, data);
        m_cacheBust++;
        m_loadState = QStringLiteral("loaded");
        emit loadStateChanged();
    });
}
