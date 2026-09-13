#include "ThreadController.h"
#include "../adapters/AdapterFactory.h"
#include "../sites/SitesController.h"

#include <QRegularExpression>

ThreadController::ThreadController(QObject *parent)
    : QAbstractListModel(parent)
{
}

int ThreadController::rowCount(const QModelIndex &parent) const
{
    if (parent.isValid()) return 0;
    return m_posts.size();
}

QVariant ThreadController::data(const QModelIndex &index, int role) const
{
    if (!index.isValid() || index.row() >= m_posts.size()) return {};
    const auto post = m_posts.at(index.row()).toMap();

    switch (role) {
    case IdRole: return post.value("id");
    case IsOpRole: return post.value("isOp");
    case NameRole: return post.value("name");
    case TripcodeRole: return post.value("tripcode");
    case CapcodeRole: return post.value("capcode");
    case SubjectRole: return post.value("subject");
    case TimestampRole: return post.value("timestamp");
    case CommentHtmlRole: return post.value("commentHtml");
    case FilesRole: return post.value("files");
    case StickyRole: return post.value("sticky");
    case ClosedRole: return post.value("closed");
    case CountryCodeRole: return post.value("countryCode");
    case CountryNameRole: return post.value("countryName");
    case BacklinksRole: return m_backlinks.value(post.value("id").toString());
    case PostRole: return post;
    default: return {};
    }
}

QHash<int, QByteArray> ThreadController::roleNames() const
{
    return {
        { IdRole, "postId" },
        { IsOpRole, "isOp" },
        { NameRole, "name" },
        { TripcodeRole, "tripcode" },
        { CapcodeRole, "capcode" },
        { SubjectRole, "subject" },
        { TimestampRole, "timestamp" },
        { CommentHtmlRole, "commentHtml" },
        { FilesRole, "files" },
        { StickyRole, "sticky" },
        { ClosedRole, "closed" },
        { CountryCodeRole, "countryCode" },
        { CountryNameRole, "countryName" },
        { BacklinksRole, "backlinks" },
        { PostRole, "post" },
    };
}

QString ThreadController::subject() const
{
    if (m_posts.isEmpty()) return {};
    return m_posts.first().toMap().value("subject").toString();
}

QVariantList ThreadController::allFiles() const
{
    QVariantList files;
    for (const auto &p : m_posts)
        files += p.toMap().value("files").toList();
    return files;
}

QStringList ThreadController::backlinksFor(const QString &postId) const
{
    return m_backlinks.value(postId);
}

int ThreadController::rowForPostId(const QString &postId) const
{
    for (int i = 0; i < m_posts.size(); ++i)
        if (m_posts.at(i).toMap().value("id").toString() == postId)
            return i;
    return -1;
}

void ThreadController::computeBacklinks()
{
    m_backlinks.clear();
    static const QRegularExpression quoteRe(QStringLiteral("href=\"quote:(\\d+)\""));

    for (const auto &pv : m_posts) {
        const auto post = pv.toMap();
        const QString replyingId = post.value("id").toString();
        const QString html = post.value("commentHtml").toString();

        auto it = quoteRe.globalMatch(html);
        while (it.hasNext())
            m_backlinks[it.next().captured(1)].append(replyingId);
    }
}

void ThreadController::load(const QString &siteId, const QString &boardCode, const QString &threadId)
{
    m_siteId = siteId;
    m_boardCode = boardCode;
    m_threadId = threadId;
    emit targetChanged();
    reload();
}

void ThreadController::reload()
{
    if (m_siteId.isEmpty() || m_boardCode.isEmpty() || m_threadId.isEmpty()) return;

    const int seq = ++m_requestSeq;
    m_loading = true;
    m_error.clear();
    emit loadingChanged();
    emit errorChanged();

    const QVariantMap site = SitesController::instance()->findSite(m_siteId);
    auto *adapter = AdapterFactory::forSite(site);
    m_threadWebUrl = adapter->threadWebUrl(site, m_boardCode, m_threadId);

    adapter->fetchThread(site, m_boardCode, m_threadId, [this, seq](QVariantMap op, QVariantList replies, QString error) {
        if (seq != m_requestSeq) return;

        m_loading = false;
        m_error = error;
        emit loadingChanged();
        emit errorChanged();

        beginResetModel();
        m_posts = QVariantList();
        if (!op.isEmpty()) m_posts.append(op);
        m_posts += replies;
        computeBacklinks();
        endResetModel();
        emit dataReset();
        emit loaded(op.value("replyCount", replies.size()).toInt());
    });
}
