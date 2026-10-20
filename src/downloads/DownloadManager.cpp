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

// ---------------------------------------------------------------------
// DownloadJobsModel
// ---------------------------------------------------------------------

DownloadJobsModel::DownloadJobsModel(DownloadManager *manager)
    : QAbstractListModel(manager)
    , m_manager(manager)
{
    connect(this, &QAbstractItemModel::rowsInserted, this, &DownloadJobsModel::countChanged);
    connect(this, &QAbstractItemModel::rowsRemoved, this, &DownloadJobsModel::countChanged);
}

int DownloadJobsModel::rowCount(const QModelIndex &parent) const
{
    return parent.isValid() ? 0 : m_manager->m_jobOrder.size();
}

QVariant DownloadJobsModel::data(const QModelIndex &index, int role) const
{
    if (!index.isValid() || index.row() >= m_manager->m_jobOrder.size())
        return {};
    const auto &job = m_manager->m_jobsById[m_manager->m_jobOrder.at(index.row())];
    switch (role) {
    case JobIdRole: return job.id;
    case LabelRole: return job.label;
    case DestDirRole: return job.destDir;
    case CreatedAtRole: return job.createdAt;
    case TotalRole: return job.items.size();
    case DoneRole: return job.done;
    case FailedRole: return job.failed;
    case ActiveRole: return job.inFlight;
    case FinishedRole: return job.finished();
    }
    return {};
}

QHash<int, QByteArray> DownloadJobsModel::roleNames() const
{
    return {
        { JobIdRole, "jobId" },
        { LabelRole, "label" },
        { DestDirRole, "destDir" },
        { CreatedAtRole, "createdAt" },
        { TotalRole, "total" },
        { DoneRole, "done" },
        { FailedRole, "failed" },
        { ActiveRole, "active" },
        { FinishedRole, "finished" },
    };
}

// ---------------------------------------------------------------------
// DownloadManager
// ---------------------------------------------------------------------

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
    , m_model(new DownloadJobsModel(this))
{
    m_flushTimer.setSingleShot(true);
    m_flushTimer.setInterval(kFlushIntervalMs);
    connect(&m_flushTimer, &QTimer::timeout, this, &DownloadManager::flush);
}

int DownloadManager::activeJobCount() const
{
    int n = 0;
    for (const auto &job : m_jobsById)
        if (!job.finished()) ++n;
    return n;
}

QString DownloadManager::statusName(Status s)
{
    switch (s) {
    case Status::Pending: return QStringLiteral("pending");
    case Status::Downloading: return QStringLiteral("downloading");
    case Status::Done: return QStringLiteral("done");
    case Status::Error: return QStringLiteral("error");
    }
    return {};
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

void DownloadManager::markDirty(const QString &jobId)
{
    m_dirty.insert(jobId);
    if (!m_flushTimer.isActive())
        m_flushTimer.start();
}

void DownloadManager::flush()
{
    const auto dirty = m_dirty;
    m_dirty.clear();
    for (const auto &jobId : dirty) {
        const int row = m_jobOrder.indexOf(jobId);
        if (row < 0) continue;
        const auto idx = m_model->index(row);
        emit m_model->dataChanged(idx, idx, { DownloadJobsModel::DoneRole, DownloadJobsModel::FailedRole,
                                              DownloadJobsModel::ActiveRole, DownloadJobsModel::FinishedRole });
        emit jobFilesChanged(jobId);
    }
    emit summaryChanged();
}

void DownloadManager::setItemStatus(const QString &jobId, int itemIndex, Status status, const QString &error)
{
    auto it = m_jobsById.find(jobId);
    if (it == m_jobsById.end() || itemIndex < 0 || itemIndex >= it->items.size()) return;
    Item &item = it->items[itemIndex];

    // Keep the summary counters in step with the transition.
    if (item.status == Status::Done) it->done--;
    if (item.status == Status::Error) it->failed--;
    if (status == Status::Done) it->done++;
    if (status == Status::Error) it->failed++;

    item.status = status;
    item.error = error;
    markDirty(jobId);
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
        Item item;
        item.url = f.value("url").toString();
        item.fileName = f.value("fileName").toString();
        job.queue.append(job.items.size());
        job.items.append(item);
    }

    // Newest job first.
    m_model->beginInsertRows({}, 0, 0);
    m_jobsById.insert(job.id, job);
    m_jobOrder.prepend(job.id);
    m_model->endInsertRows();
    emit summaryChanged();

    pump(job.id);
    return job.id;
}

void DownloadManager::pump(const QString &jobId)
{
    auto it = m_jobsById.find(jobId);
    if (it == m_jobsById.end()) return;

    while (it->inFlight < kConcurrency && !it->queue.isEmpty()) {
        const int itemIndex = it->queue.takeFirst();
        it->inFlight++;

        const QString url = it->items.at(itemIndex).url;
        const QString destPath = uniqueDestPath(it->destDir, it->items.at(itemIndex).fileName);

        setItemStatus(jobId, itemIndex, Status::Downloading);

        NetworkClient::instance()->get(url, [this, jobId, itemIndex, destPath](bool ok, const QByteArray &data, const QString &error) {
            if (!m_jobsById.contains(jobId)) return;
            if (ok) {
                QFile out(destPath);
                if (out.open(QIODevice::WriteOnly)) {
                    out.write(data);
                    setItemStatus(jobId, itemIndex, Status::Done);
                } else {
                    setItemStatus(jobId, itemIndex, Status::Error, QStringLiteral("could not write file"));
                }
            } else {
                setItemStatus(jobId, itemIndex, Status::Error, error);
            }

            m_jobsById[jobId].inFlight--;
            pump(jobId);
        });
    }
}

QVariantList DownloadManager::jobFiles(const QString &jobId) const
{
    QVariantList out;
    const auto it = m_jobsById.constFind(jobId);
    if (it == m_jobsById.constEnd()) return out;
    out.reserve(it->items.size());
    for (const auto &item : it->items) {
        QVariantMap m;
        m["fileName"] = item.fileName;
        m["status"] = statusName(item.status);
        m["error"] = item.error;
        out.append(m);
    }
    return out;
}

void DownloadManager::retryFailed(const QString &jobId)
{
    auto it = m_jobsById.find(jobId);
    if (it == m_jobsById.end()) return;
    for (int i = 0; i < it->items.size(); ++i) {
        if (it->items.at(i).status == Status::Error) {
            setItemStatus(jobId, i, Status::Pending);
            it->queue.append(i);
        }
    }
    pump(jobId);
}

void DownloadManager::removeJob(const QString &jobId)
{
    const int row = m_jobOrder.indexOf(jobId);
    if (row < 0 || !m_jobsById[jobId].finished()) return;
    m_model->beginRemoveRows({}, row, row);
    m_jobOrder.removeAt(row);
    m_jobsById.remove(jobId);
    m_dirty.remove(jobId);
    m_model->endRemoveRows();
    emit summaryChanged();
}

void DownloadManager::clearFinished()
{
    for (int row = m_jobOrder.size() - 1; row >= 0; --row) {
        const QString id = m_jobOrder.at(row);
        if (!m_jobsById[id].finished()) continue;
        m_model->beginRemoveRows({}, row, row);
        m_jobOrder.removeAt(row);
        m_jobsById.remove(id);
        m_dirty.remove(id);
        m_model->endRemoveRows();
    }
    emit summaryChanged();
}

void DownloadManager::revealInFolder(const QString &filePath) const
{
    QDesktopServices::openUrl(QUrl::fromLocalFile(QFileInfo(filePath).absolutePath()));
}

void DownloadManager::openDirectory(const QString &dirPath) const
{
    QDesktopServices::openUrl(QUrl::fromLocalFile(dirPath));
}
