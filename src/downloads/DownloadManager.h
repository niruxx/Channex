#pragma once

#include <QMap>
#include <QObject>
#include <QQmlEngine>
#include <QStringList>
#include <QVariantList>
#include <QVariantMap>

// Mirrors src/store/useDownloadsStore.ts + src/lib/download.ts: a
// fixed-size worker pool (CONCURRENCY = 4) pulls from a shared cursor
// over the requested files, fetches each fully into memory, and writes
// it to disk with collision-safe naming (" (1)", " (2)", ...). Session
// state only - jobs are not persisted across restarts, same as upstream.
class DownloadManager : public QObject
{
    Q_OBJECT
    QML_ELEMENT
    QML_SINGLETON

    Q_PROPERTY(QVariantList jobs READ jobs NOTIFY jobsChanged)

public:
    // Constructor stays private: a public one makes this look default-
    // constructible to QML_SINGLETON's factory detection, which can then
    // bypass create() and hand QML a disconnected instance.
    static DownloadManager *create(QQmlEngine *, QJSEngine *);
    static DownloadManager *instance();

    QVariantList jobs() const { return m_jobs; }

    // files: list of {url, fileName}
    Q_INVOKABLE QString startJob(const QString &label, const QString &destDir, const QVariantList &files);
    Q_INVOKABLE void revealInFolder(const QString &filePath) const;
    Q_INVOKABLE void openDirectory(const QString &dirPath) const;

signals:
    void jobsChanged();

private:
    explicit DownloadManager(QObject *parent = nullptr);

    static const int kConcurrency = 4;

    struct Job {
        QString id, label, destDir;
        qint64 createdAt = 0;
        QVariantList items; // {id, url, fileName, status, error}
        int cursor = 0;
        int inFlight = 0;
    };

    void pump(const QString &jobId);
    void setItemStatus(const QString &jobId, int itemIndex, const QString &status, const QString &error = {});
    void emitJobsChanged();
    QString uniqueDestPath(const QString &destDir, const QString &fileName) const;

    QMap<QString, Job> m_jobsById;
    QVariantList m_jobs; // display copy, kept in job insertion order
    QStringList m_jobOrder;
};
