#include "CatalogController.h"
#include "../adapters/AdapterFactory.h"
#include "../sites/SitesController.h"

#include <QRegularExpression>
#include <algorithm>

CatalogController::CatalogController(QObject *parent)
    : QAbstractListModel(parent)
{
}

int CatalogController::rowCount(const QModelIndex &parent) const
{
    if (parent.isValid()) return 0;
    return m_view.size();
}

QVariant CatalogController::data(const QModelIndex &index, int role) const
{
    if (!index.isValid() || index.row() >= m_view.size()) return {};
    const auto post = m_view.at(index.row()).toMap();
    const auto files = post.value("files").toList();
    const auto firstFile = files.isEmpty() ? QVariantMap() : files.first().toMap();

    switch (role) {
    case IdRole: return post.value("id");
    case ThreadIdRole: return post.value("threadId");
    case SubjectRole: return post.value("subject");
    case CommentHtmlRole: return post.value("commentHtml");
    case NameRole: return post.value("name");
    case TimestampRole: return post.value("timestamp");
    case StickyRole: return post.value("sticky");
    case ClosedRole: return post.value("closed");
    case ReplyCountRole: return post.value("replyCount");
    case ImageCountRole: return post.value("imageCount");
    case ThumbUrlRole: return firstFile.value("thumbUrl");
    case IsVideoRole: return firstFile.value("isVideo");
    case SpoilerRole: return firstFile.value("spoiler");
    case FileCountRole: return files.size();
    case PostRole: return post;
    default: return {};
    }
}

QHash<int, QByteArray> CatalogController::roleNames() const
{
    return {
        { IdRole, "postId" },
        { ThreadIdRole, "threadId" },
        { SubjectRole, "subject" },
        { CommentHtmlRole, "commentHtml" },
        { NameRole, "name" },
        { TimestampRole, "timestamp" },
        { StickyRole, "sticky" },
        { ClosedRole, "closed" },
        { ReplyCountRole, "replyCount" },
        { ImageCountRole, "imageCount" },
        { ThumbUrlRole, "thumbUrl" },
        { IsVideoRole, "isVideo" },
        { SpoilerRole, "spoiler" },
        { FileCountRole, "fileCount" },
        { PostRole, "post" },
    };
}

void CatalogController::setSearchQuery(const QString &q)
{
    if (m_searchQuery == q) return;
    m_searchQuery = q;
    emit searchQueryChanged();
    recomputeView();
}

void CatalogController::setSortMode(const QString &mode)
{
    if (m_sortMode == mode) return;
    m_sortMode = mode;
    emit sortModeChanged();
    recomputeView();
}

QString CatalogController::plainTextExcerpt(const QVariantMap &post)
{
    QString html = post.value("commentHtml").toString();
    html.remove(QRegularExpression(QStringLiteral("<[^>]*>")));
    return html;
}

void CatalogController::recomputeView()
{
    QVariantList filtered;
    if (m_searchQuery.trimmed().isEmpty()) {
        filtered = m_raw;
    } else {
        const QString needle = m_searchQuery.trimmed();
        for (const auto &v : m_raw) {
            const auto post = v.toMap();
            if (post.value("subject").toString().contains(needle, Qt::CaseInsensitive) ||
                post.value("threadId").toString().contains(needle, Qt::CaseInsensitive) ||
                plainTextExcerpt(post).contains(needle, Qt::CaseInsensitive))
                filtered.append(v);
        }
    }

    std::stable_sort(filtered.begin(), filtered.end(), [this](const QVariant &av, const QVariant &bv) {
        const auto a = av.toMap();
        const auto b = bv.toMap();
        const bool aSticky = a.value("sticky").toBool();
        const bool bSticky = b.value("sticky").toBool();
        if (aSticky != bSticky) return aSticky;

        if (m_sortMode == QStringLiteral("replies"))
            return a.value("replyCount").toInt() > b.value("replyCount").toInt();
        if (m_sortMode == QStringLiteral("images"))
            return a.value("imageCount").toInt() > b.value("imageCount").toInt();
        if (m_sortMode == QStringLiteral("oldest"))
            return a.value("timestamp").toLongLong() < b.value("timestamp").toLongLong();
        // "bump" (default): latest first
        return a.value("timestamp").toLongLong() > b.value("timestamp").toLongLong();
    });

    beginResetModel();
    m_view = filtered;
    endResetModel();
    emit dataReset();
}

void CatalogController::load(const QString &siteId, const QString &boardCode)
{
    m_siteId = siteId;
    m_boardCode = boardCode;
    emit targetChanged();
    reload();
}

void CatalogController::reload()
{
    if (m_siteId.isEmpty() || m_boardCode.isEmpty()) return;

    const int seq = ++m_requestSeq;
    m_loading = true;
    m_error.clear();
    emit loadingChanged();
    emit errorChanged();

    const QVariantMap site = SitesController::instance()->findSite(m_siteId);
    AdapterFactory::forSite(site)->fetchCatalog(site, m_boardCode, [this, seq](QVariantList posts, QString error) {
        if (seq != m_requestSeq) return; // stale response from a nav change mid-flight

        m_loading = false;
        m_error = error;
        m_raw = posts;
        emit loadingChanged();
        emit errorChanged();
        recomputeView();
    });
}
