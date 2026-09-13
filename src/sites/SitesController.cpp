#include "SitesController.h"
#include "../adapters/AdapterFactory.h"
#include "../settings/SettingsManager.h"

#include <algorithm>

namespace {

QVariantMap board(const QString &code, const QString &title)
{
    QVariantMap b;
    b["code"] = code;
    b["title"] = title;
    b["description"] = QString();
    b["nsfw"] = false;
    return b;
}

QVariantMap makeSite(const QString &id, const QString &name, const QString &schema,
                      const QString &mediaLayout, const QString &apiOrigin,
                      const QString &siteOrigin, const QString &mediaOrigin,
                      const QString &accent, bool nsfw, const QString &postEngine,
                      const QVariantList &defaultBoards, const QString &favicon = {})
{
    QVariantMap s;
    s["id"] = id;
    s["name"] = name;
    s["schema"] = schema;
    s["mediaLayout"] = mediaLayout;
    s["apiOrigin"] = apiOrigin;
    s["siteOrigin"] = siteOrigin;
    s["mediaOrigin"] = mediaOrigin;
    s["accent"] = accent;
    s["isCustom"] = false;
    s["nsfw"] = nsfw;
    s["postEngine"] = postEngine;
    s["defaultBoards"] = defaultBoards;
    s["favicon"] = favicon;
    return s;
}

} // namespace

namespace { SitesController *g_instance = nullptr; }

SitesController *SitesController::create(QQmlEngine *, QJSEngine *)
{
    if (!g_instance)
        g_instance = new SitesController();
    return g_instance;
}

SitesController *SitesController::instance()
{
    if (!g_instance)
        g_instance = new SitesController();
    return g_instance;
}

SitesController::SitesController(QObject *parent)
    : QObject(parent)
{
}

QVariantList SitesController::presetSites() const
{
    QVariantList fourchanBoards = {
        board("g", "Technology"), board("v", "Video Games"), board("a", "Anime & Manga"),
        board("tv", "Television & Film"), board("mu", "Music"), board("fit", "Fitness"),
        board("ck", "Food & Cooking"), board("diy", "Do It Yourself"), board("sci", "Science & Math"),
        board("lit", "Literature"), board("out", "Outdoors"), board("co", "Comics & Cartoons"),
        board("fa", "Fashion"), board("int", "International"), board("wg", "Wallpapers"),
        board("biz", "Business & Finance"),
    };

    return {
        makeSite("4chan", "4chan", "yotsuba", "flat-cdn",
                 "https://a.4cdn.org", "https://boards.4chan.org", "https://i.4cdn.org",
                 "#3fae6a", false, "external", fourchanBoards, "https://s.4cdn.org/image/favicon.ico"),
        makeSite("8kun", "8kun", "yotsuba", "file-store",
                 "https://8kun.top", "https://8kun.top", "https://media.8kun.top",
                 "#c9a227", true, "lynxchan", {}),
        makeSite("8chan-moe", "8chan.moe", "lynxchan", "board-dirs",
                 "https://8chan.moe", "https://8chan.moe", "https://8chan.moe",
                 "#8b5cf6", true, "lynxchan", {}, "https://8chan.moe/favicon.ico"),
        makeSite("lainchan", "Lainchan", "yotsuba", "board-dirs",
                 "https://lainchan.org", "https://lainchan.org", "https://lainchan.org",
                 "#4fc3c7", false, "external", {}, "https://lainchan.org/favicon.ico"),
    };
}

QVariantList SitesController::sites() const
{
    QVariantList all = presetSites();
    for (const auto &v : SettingsManager::instance()->customSites())
        all.append(v);
    return all;
}

QVariantMap SitesController::findSite(const QString &id) const
{
    for (const auto &v : sites()) {
        const auto m = v.toMap();
        if (m.value("id").toString() == id)
            return m;
    }
    return {};
}

void SitesController::setCurrentSiteId(const QString &id)
{
    if (m_currentSiteId == id) return;
    m_currentSiteId = id;
    emit currentSiteIdChanged();
    if (!m_boardState.contains(id))
        loadBoards(id);
    else
        emit boardsChanged();
}

QVariantList SitesController::currentBoards() const
{
    return m_boardState.value(m_currentSiteId).boards;
}

bool SitesController::boardsLoading() const
{
    return m_boardState.value(m_currentSiteId).loading;
}

QString SitesController::boardsError() const
{
    return m_boardState.value(m_currentSiteId).error;
}

void SitesController::mergeDefaultBoards(const QString &siteId, QVariantList &boards) const
{
    const QVariantMap site = findSite(siteId);
    QMap<QString, QVariantMap> byCode;
    for (const auto &v : boards)
        byCode[v.toMap().value("code").toString()] = v.toMap();
    for (const auto &v : site.value("defaultBoards").toList()) {
        const auto b = v.toMap();
        if (!byCode.contains(b.value("code").toString()))
            byCode[b.value("code").toString()] = b;
    }
    boards = QVariantList();
    for (auto it = byCode.constBegin(); it != byCode.constEnd(); ++it)
        boards.append(it.value());
    std::sort(boards.begin(), boards.end(), [](const QVariant &a, const QVariant &b) {
        return a.toMap().value("code").toString() < b.toMap().value("code").toString();
    });
}

void SitesController::loadBoards(const QString &siteId)
{
    const QVariantMap site = findSite(siteId);
    if (site.isEmpty()) return;

    m_boardState[siteId].loading = true;
    m_boardState[siteId].error.clear();
    if (siteId == m_currentSiteId)
        emit boardsChanged();

    AdapterFactory::forSite(site)->fetchBoards(site, [this, siteId](QVariantList boards, QString error) {
        mergeDefaultBoards(siteId, boards);
        auto &state = m_boardState[siteId];
        state.boards = boards;
        state.loading = false;
        state.error = error;
        if (siteId == m_currentSiteId)
            emit boardsChanged();
    });
}

void SitesController::addCustomSite(const QVariantMap &input)
{
    QString origin = input.value("origin").toString();
    while (origin.endsWith('/'))
        origin.chop(1);
    QString mediaOrigin = input.value("mediaOrigin").toString();
    if (mediaOrigin.isEmpty()) mediaOrigin = origin;
    while (mediaOrigin.endsWith('/'))
        mediaOrigin.chop(1);

    QVariantMap site;
    site["id"] = input.value("id");
    site["name"] = input.value("name");
    site["schema"] = input.value("schema", "yotsuba");
    site["mediaLayout"] = input.value("mediaLayout", "board-dirs");
    site["apiOrigin"] = origin;
    site["siteOrigin"] = origin;
    site["mediaOrigin"] = mediaOrigin;
    site["accent"] = input.value("accent", "#6ee7c9");
    site["isCustom"] = true;
    site["nsfw"] = input.value("nsfw", true);
    site["postEngine"] = input.value("postEngine", "external");
    site["defaultBoards"] = QVariantList();
    site["favicon"] = origin + "/favicon.ico";

    auto list = SettingsManager::instance()->customSites();
    list.append(site);
    SettingsManager::instance()->setCustomSites(list);
    emit sitesChanged();
}

void SitesController::removeCustomSite(const QString &id)
{
    auto list = SettingsManager::instance()->customSites();
    for (int i = 0; i < list.size(); ++i) {
        if (list.at(i).toMap().value("id").toString() == id) {
            list.removeAt(i);
            break;
        }
    }
    SettingsManager::instance()->setCustomSites(list);
    emit sitesChanged();
}
