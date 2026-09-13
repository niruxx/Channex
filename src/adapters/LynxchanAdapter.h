#pragma once

#include "ChanAdapter.h"

// LynxChan JSON API (8chan.moe / 8kun-family boards) - mirrors
// src/lib/adapters/lynxchan.ts.
class LynxchanAdapter : public ChanAdapter
{
    Q_OBJECT
public:
    using ChanAdapter::ChanAdapter;

    void fetchBoards(const QVariantMap &site, BoardsCallback cb) override;
    void fetchCatalog(const QVariantMap &site, const QString &boardCode, PostsCallback cb) override;
    void fetchThread(const QVariantMap &site, const QString &boardCode, const QString &threadId, ThreadCallback cb) override;
    QString threadWebUrl(const QVariantMap &site, const QString &boardCode, const QString &threadId) const override;

private:
    static QString resolveUrl(const QVariantMap &site, const QString &path);
    static QVariantMap fileOf(const QVariantMap &site, const QVariantMap &rawFile);
    static QString commentHtmlOf(const QVariantMap &raw);
};
