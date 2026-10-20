#pragma once

#include <QAbstractListModel>
#include <QHash>
#include <QList>
#include <QObject>
#include <QQmlEngine>
#include <QSet>
#include <QStringList>
#include <QTimer>
#include <QVariantList>

class DownloadManager;

// One row per download job, exposing only summary roles (label, counts,
// destination). Rows update in place via dataChanged, so delegates keep
// living across progress updates instead of the whole list being
// rebuilt - per-file details are fetched on demand with
// DownloadManager::jobFiles() only when a job's file preview is open.
class DownloadJobsModel : public QAbstractListModel
{
    Q_OBJECT
    Q_PROPERTY(int count READ rowCount NOTIFY countChanged)

public:
    enum Role {
        JobIdRole = Qt::UserRole + 1,
        LabelRole,
        DestDirRole,
        CreatedAtRole,
        TotalRole,
        DoneRole,
        FailedRole,
        ActiveRole,
        FinishedRole,
    };

    explicit DownloadJobsModel(DownloadManager *manager);

    int rowCount(const QModelIndex &parent = {}) const override;
    QVariant data(const QModelIndex &index, int role) const override;
    QHash<int, QByteArray> roleNames() const override;

signals:
    void countChanged();

private:
    friend class DownloadManager;
    DownloadManager *m_manager;
};

// Mirrors src/store/useDownloadsStore.ts + src/lib/download.ts: a
// fixed-size worker pool (CONCURRENCY = 4) per job pulls from a queue of
// the requested files, fetches each fully into memory, and writes it to
// disk with collision-safe naming (" (1)", " (2)", ...). Session state
// only - jobs are not persisted across restarts, same as upstream.
//
// Status changes are batched: affected jobs are marked dirty and
// flushed to QML at most every kFlushIntervalMs, so a 300-file job
// doesn't push 600 separate UI updates.
class DownloadManager : public QObject
{
    Q_OBJECT
    QML_ELEMENT
    QML_SINGLETON

    Q_PROPERTY(DownloadJobsModel *jobs READ jobs CONSTANT)
    Q_PROPERTY(int jobCount READ jobCount NOTIFY summaryChanged)
    Q_PROPERTY(int activeJobCount READ activeJobCount NOTIFY summaryChanged)

public:
    // Constructor stays private: a public one makes this look default-
    // constructible to QML_SINGLETON's factory detection, which can then
    // bypass create() and hand QML a disconnected instance.
    static DownloadManager *create(QQmlEngine *, QJSEngine *);
    static DownloadManager *instance();

    DownloadJobsModel *jobs() const { return m_model; }
    int jobCount() const { return m_jobOrder.size(); }
    int activeJobCount() const;

    // files: list of {url, fileName}
    Q_INVOKABLE QString startJob(const QString &label, const QString &destDir, const QVariantList &files);
    // Per-file rows {fileName, status, error} for one job - only called
    // while that job's file preview is expanded.
    Q_INVOKABLE QVariantList jobFiles(const QString &jobId) const;
    Q_INVOKABLE void retryFailed(const QString &jobId);
    // History only: these drop finished jobs from the list and never
    // touch the downloaded files on disk. Jobs still running are kept.
    Q_INVOKABLE void removeJob(const QString &jobId);
    Q_INVOKABLE void clearFinished();
    Q_INVOKABLE void revealInFolder(const QString &filePath) const;
    Q_INVOKABLE void openDirectory(const QString &dirPath) const;

signals:
    void summaryChanged();
    // Emitted (batched) when a job's per-file statuses changed; an open
    // file preview re-reads jobFiles() for that job.
    void jobFilesChanged(const QString &jobId);

private:
    friend class DownloadJobsModel;
    explicit DownloadManager(QObject *parent = nullptr);

    static const int kConcurrency = 4;
    static const int kFlushIntervalMs = 120;

    enum class Status { Pending, Downloading, Done, Error };

    struct Item {
        QString url, fileName, error;
        Status status = Status::Pending;
    };

    struct Job {
        QString id, label, destDir;
        qint64 createdAt = 0;
        QList<Item> items;
        QList<int> queue; // indexes of items still to fetch
        int inFlight = 0;
        int done = 0;
        int failed = 0;
        bool finished() const { return queue.isEmpty() && inFlight == 0; }
    };

    void pump(const QString &jobId);
    void setItemStatus(const QString &jobId, int itemIndex, Status status, const QString &error = {});
    void markDirty(const QString &jobId);
    void flush();
    QString uniqueDestPath(const QString &destDir, const QString &fileName) const;
    static QString statusName(Status s);

    QHash<QString, Job> m_jobsById;
    QStringList m_jobOrder; // display order (insertion)
    QSet<QString> m_dirty;
    QTimer m_flushTimer;
    DownloadJobsModel *m_model;
};
