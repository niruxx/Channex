#include "DownloadManager.h"
#include "../network/NetworkClient.h"

#include <QDesktopServices>
#include <QDateTime>
#include <QDir>
#include <QFile>
#include <QFileInfo>
#include <QUrl>
#include <QUuid>

namespace { DownloadManager *g_instance = nullptr; }

DownloadManager *DownloadManager::create(QQmlEngine *, QJSEngine *)
{
    if (!g_instance)
        g_instance = new DownloadManager();
    return g_instance;
}

DownloadManager *DownloadManager::instance()
{
    if (!g_instance)
        g_instance = new DownloadManager();
    return g_instance;
}

DownloadManager::DownloadManager(QObject *parent)
    : QObject(parent)
{
}

QString DownloadManager::uniqueDestPath(const QString &destDir, const QString &fileName) const
{
    QFileInfo fi(fileName);
    const QString base = fi.completeBaseName();
    const QString ext = fi.suffix();

    QString candidate = destDir + '/' + fileName;
    int n = 1;
    while (QFile::exists(candidate)) {
        candidate = ext.isEmpty()
            ? QStringLiteral("%1/%2 (%3)").arg(destDir, base).arg(n)
            : QStringLiteral("%1/%2 (%3).%4").arg(destDir, base).arg(n).arg(ext);
        ++n;
    }
    return candidate;
}

void DownloadManager::emitJobsChanged()
{
    QVariantList display;
    for (const auto &id : m_jobOrder) {
        const auto &job = m_jobsById[id];
        QVariantMap m;
        m["id"] = job.id;
        m["label"] = job.label;
        m["destDir"] = job.destDir;
        m["createdAt"] = job.createdAt;
        m["items"] = job.items;
        display.append(m);
    }
    m_jobs = display;
    emit jobsChanged();
}

void DownloadManager::setItemStatus(const QString &jobId, int itemIndex, const QString &status, const QString &error)
{
    auto &job = m_jobsById[jobId];
    if (itemIndex < 0 || itemIndex >= job.items.size()) return;
    QVariantMap item = job.items.at(itemIndex).toMap();
    item["status"] = status;
    if (!error.isEmpty()) item["error"] = error;
    job.items[itemIndex] = item;
    emitJobsChanged();
}

QString DownloadManager::startJob(const QString &label, const QString &destDir, const QVariantList &files)
{
    QDir().mkpath(destDir);

    Job job;
    job.id = QUuid::createUuid().toString(QUuid::WithoutBraces);
    job.label = label;
    job.destDir = destDir;
    job.createdAt = QDateTime::currentMSecsSinceEpoch();

    for (const auto &fv : files) {
        const auto f = fv.toMap();
        QVariantMap item;
        item["id"] = QUuid::createUuid().toString(QUuid::WithoutBraces);
        item["url"] = f.value("url");
        item["fileName"] = f.value("fileName");
        item["status"] = QStringLiteral("pending");
        job.items.append(item);
    }

    m_jobsById[job.id] = job;
    m_jobOrder.append(job.id);
    emitJobsChanged();

    pump(job.id);
    return job.id;
}

void DownloadManager::pump(const QString &jobId)
{
    auto &job = m_jobsById[jobId];
    while (job.inFlight < kConcurrency && job.cursor < job.items.size()) {
        const int itemIndex = job.cursor++;
        job.inFlight++;

        const auto item = job.items.at(itemIndex).toMap();
        const QString url = item.value("url").toString();
        const QString fileName = item.value("fileName").toString();
        const QString destPath = uniqueDestPath(job.destDir, fileName);

        setItemStatus(jobId, itemIndex, QStringLiteral("downloading"));

        NetworkClient::instance()->get(url, [this, jobId, itemIndex, destPath](bool ok, const QByteArray &data, const QString &error) {
            if (ok) {
                QFile out(destPath);
                if (out.open(QIODevice::WriteOnly)) {
                    out.write(data);
                    setItemStatus(jobId, itemIndex, QStringLiteral("done"));
                } else {
                    setItemStatus(jobId, itemIndex, QStringLiteral("error"), QStringLiteral("could not write file"));
                }
            } else {
                setItemStatus(jobId, itemIndex, QStringLiteral("error"), error);
            }

            if (m_jobsById.contains(jobId)) {
                m_jobsById[jobId].inFlight--;
                pump(jobId);
            }
        });
    }
}

void DownloadManager::revealInFolder(const QString &filePath) const
{
    QDesktopServices::openUrl(QUrl::fromLocalFile(QFileInfo(filePath).absolutePath()));
}

void DownloadManager::openDirectory(const QString &dirPath) const
{
    QDesktopServices::openUrl(QUrl::fromLocalFile(dirPath));
}
