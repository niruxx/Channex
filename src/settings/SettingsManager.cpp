#include "SettingsManager.h"

#include <QCoreApplication>
#include <QDir>
#include <QFile>
#include <QJsonArray>
#include <QJsonDocument>
#include <QJsonObject>
#include <QStandardPaths>

namespace {
SettingsManager *g_instance = nullptr;
}

SettingsManager *SettingsManager::create(QQmlEngine *, QJSEngine *)
{
    // Ownership is taken by the QML engine; g_instance lets C++ code
    // (adapters, controllers) reach the same object without re-parsing.
    if (!g_instance)
        g_instance = new SettingsManager();
    return g_instance;
}

SettingsManager *SettingsManager::instance()
{
    if (!g_instance)
        g_instance = new SettingsManager();
    return g_instance;
}

SettingsManager::SettingsManager(QObject *parent)
    : QObject(parent)
{
}

QString SettingsManager::storePath() const
{
    const QString dir = QStandardPaths::writableLocation(QStandardPaths::AppDataLocation);
    QDir().mkpath(dir);
    return dir + QStringLiteral("/channex-qt-data.json");
}

void SettingsManager::hydrate()
{
    QFile file(storePath());
    if (file.open(QIODevice::ReadOnly)) {
        const auto doc = QJsonDocument::fromJson(file.readAll());
        const QJsonObject obj = doc.object();

        m_downloadDir = obj.value("downloadDir").toString();
        m_blurNsfw = obj.value("blurNsfw").toBool(true);
        m_hideNsfwSites = obj.value("hideNsfwSites").toBool(false);
        m_theme = obj.value("theme").toString("dark");
        m_accentColor = obj.value("accentColor").toString("#6ee7c9");
        m_backgroundTheme = obj.value("backgroundTheme").toString("none");
        m_catalogViewMode = obj.value("catalogViewMode").toString("grid");
        m_replyDisplayStyle = obj.value("replyDisplayStyle").toString("standard");
        m_muteWebmsByDefault = obj.value("muteWebmsByDefault").toBool(true);
        m_birthdayHats = obj.value("birthdayHats").toBool(true);
        m_hasCompletedOnboarding = obj.value("hasCompletedOnboarding").toBool(false);

        m_customSites.clear();
        for (const auto &v : obj.value("customSites").toArray())
            m_customSites.append(v.toObject().toVariantMap());

        m_hiddenBoards = obj.value("hiddenBoards").toObject().toVariantMap();
    }

    m_hydrated = true;
    emit hydratedChanged();
    emit downloadDirChanged();
    emit blurNsfwChanged();
    emit hideNsfwSitesChanged();
    emit themeChanged();
    emit accentColorChanged();
    emit backgroundThemeChanged();
    emit catalogViewModeChanged();
    emit replyDisplayStyleChanged();
    emit muteWebmsByDefaultChanged();
    emit birthdayHatsChanged();
    emit hasCompletedOnboardingChanged();
    emit customSitesChanged();
    emit hiddenBoardsChanged();
}

void SettingsManager::save() const
{
    // Read-modify-write: BookmarksController persists the "bookmarks"
    // key in this same file, so a naive overwrite here would wipe it
    // out every time a setting changes.
    QJsonObject obj;
    QFile readFile(storePath());
    if (readFile.open(QIODevice::ReadOnly)) {
        obj = QJsonDocument::fromJson(readFile.readAll()).object();
        readFile.close();
    }

    obj["downloadDir"] = m_downloadDir;
    obj["blurNsfw"] = m_blurNsfw;
    obj["hideNsfwSites"] = m_hideNsfwSites;
    obj["theme"] = m_theme;
    obj["accentColor"] = m_accentColor;
    obj["backgroundTheme"] = m_backgroundTheme;
    obj["catalogViewMode"] = m_catalogViewMode;
    obj["replyDisplayStyle"] = m_replyDisplayStyle;
    obj["muteWebmsByDefault"] = m_muteWebmsByDefault;
    obj["birthdayHats"] = m_birthdayHats;
    obj["hasCompletedOnboarding"] = m_hasCompletedOnboarding;

    QJsonArray customSites;
    for (const auto &v : m_customSites)
        customSites.append(QJsonObject::fromVariantMap(v.toMap()));
    obj["customSites"] = customSites;

    obj["hiddenBoards"] = QJsonObject::fromVariantMap(m_hiddenBoards);

    QFile file(storePath());
    if (file.open(QIODevice::WriteOnly | QIODevice::Truncate))
        file.write(QJsonDocument(obj).toJson(QJsonDocument::Indented));
}

void SettingsManager::setDownloadDirFromUrl(const QUrl &url)
{
    setDownloadDir(url.toLocalFile());
}

void SettingsManager::setDownloadDir(const QString &v)
{
    if (m_downloadDir == v) return;
    m_downloadDir = v;
    emit downloadDirChanged();
    save();
}

void SettingsManager::setBlurNsfw(bool v)
{
    if (m_blurNsfw == v) return;
    m_blurNsfw = v;
    emit blurNsfwChanged();
    save();
}

void SettingsManager::setHideNsfwSites(bool v)
{
    if (m_hideNsfwSites == v) return;
    m_hideNsfwSites = v;
    emit hideNsfwSitesChanged();
    save();
}

void SettingsManager::setTheme(const QString &v)
{
    if (m_theme == v) return;
    m_theme = v;
    emit themeChanged();
    save();
}

void SettingsManager::setAccentColor(const QString &v)
{
    if (m_accentColor == v) return;
    m_accentColor = v;
    emit accentColorChanged();
    save();
}

void SettingsManager::setBackgroundTheme(const QString &v)
{
    if (m_backgroundTheme == v) return;
    m_backgroundTheme = v;
    emit backgroundThemeChanged();
    save();
}

void SettingsManager::setCatalogViewMode(const QString &v)
{
    if (m_catalogViewMode == v) return;
    m_catalogViewMode = v;
    emit catalogViewModeChanged();
    save();
}

void SettingsManager::setReplyDisplayStyle(const QString &v)
{
    if (m_replyDisplayStyle == v) return;
    m_replyDisplayStyle = v;
    emit replyDisplayStyleChanged();
    save();
}

void SettingsManager::setMuteWebmsByDefault(bool v)
{
    if (m_muteWebmsByDefault == v) return;
    m_muteWebmsByDefault = v;
    emit muteWebmsByDefaultChanged();
    save();
}

void SettingsManager::setBirthdayHats(bool v)
{
    if (m_birthdayHats == v) return;
    m_birthdayHats = v;
    emit birthdayHatsChanged();
    save();
}

void SettingsManager::setHasCompletedOnboarding(bool v)
{
    if (m_hasCompletedOnboarding == v) return;
    m_hasCompletedOnboarding = v;
    emit hasCompletedOnboardingChanged();
    save();
}

void SettingsManager::setCustomSites(const QVariantList &v)
{
    m_customSites = v;
    emit customSitesChanged();
    save();
}

void SettingsManager::setHiddenBoards(const QVariantMap &v)
{
    m_hiddenBoards = v;
    emit hiddenBoardsChanged();
    save();
}

void SettingsManager::setBoardHidden(const QString &siteId, const QString &boardCode, bool hidden)
{
    QVariantList list = m_hiddenBoards.value(siteId).toList();
    const int idx = list.indexOf(boardCode);
    if (hidden && idx < 0)
        list.append(boardCode);
    else if (!hidden && idx >= 0)
        list.removeAt(idx);
    else
        return;

    m_hiddenBoards[siteId] = list;
    emit hiddenBoardsChanged();
    save();
}
