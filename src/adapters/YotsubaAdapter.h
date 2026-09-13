#pragma once

#include "ChanAdapter.h"

// Handles 4chan's native JSON API and the vichan/Lainchan/8kun family
// that copies its shape ("Yotsuba" is 4chan's default skin/engine
// name) - mirrors src/lib/adapters/yotsuba.ts.
class YotsubaAdapter : public ChanAdapter
{
    Q_OBJECT
public:
    using ChanAdapter::ChanAdapter;

    void fetchBoards(const QVariantMap &site, BoardsCallback cb) override;
    void fetchCatalog(const QVariantMap &site, const QString &boardCode, PostsCallback cb) override;
    void fetchThread(const QVariantMap &site, const QString &boardCode, const QString &threadId, ThreadCallback cb) override;
    QString threadWebUrl(const QVariantMap &site, const QString &boardCode, const QString &threadId) const override;

private:
    static QVariantMap normalizePost(const QVariantMap &site, const QString &boardCode, const QVariantMap &raw);
};
