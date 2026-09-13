#pragma once

#include <QObject>
#include <QQmlEngine>
#include <QString>
#include <QUrl>
#include <QVariantList>

// Mirrors the persisted-settings schema of the Tauri app's
// imageboarder-data.json, but stored in its own file
// (channex-qt-data.json) since this is a parallel placeholder build.
class SettingsManager : public QObject
{
    Q_OBJECT
    QML_ELEMENT
    QML_SINGLETON

    Q_PROPERTY(QString downloadDir READ downloadDir WRITE setDownloadDir NOTIFY downloadDirChanged)
    Q_PROPERTY(bool blurNsfw READ blurNsfw WRITE setBlurNsfw NOTIFY blurNsfwChanged)
    Q_PROPERTY(bool hideNsfwSites READ hideNsfwSites WRITE setHideNsfwSites NOTIFY hideNsfwSitesChanged)
    Q_PROPERTY(QString theme READ theme WRITE setTheme NOTIFY themeChanged)
    Q_PROPERTY(QString accentColor READ accentColor WRITE setAccentColor NOTIFY accentColorChanged)
    Q_PROPERTY(QString backgroundTheme READ backgroundTheme WRITE setBackgroundTheme NOTIFY backgroundThemeChanged)
    Q_PROPERTY(QString catalogViewMode READ catalogViewMode WRITE setCatalogViewMode NOTIFY catalogViewModeChanged)
    Q_PROPERTY(bool muteWebmsByDefault READ muteWebmsByDefault WRITE setMuteWebmsByDefault NOTIFY muteWebmsByDefaultChanged)
    Q_PROPERTY(bool birthdayHats READ birthdayHats WRITE setBirthdayHats NOTIFY birthdayHatsChanged)
    Q_PROPERTY(bool hasCompletedOnboarding READ hasCompletedOnboarding WRITE setHasCompletedOnboarding NOTIFY hasCompletedOnboardingChanged)
    Q_PROPERTY(QVariantList customSites READ customSites WRITE setCustomSites NOTIFY customSitesChanged)
    Q_PROPERTY(bool hydrated READ hydrated NOTIFY hydratedChanged)

public:
    // A public constructor here would make this type look default-
    // constructible to Qt's QML_SINGLETON factory-detection, which can
    // then bypass create() entirely and hand QML a second, never-hydrated
    // instance disconnected from the one C++ code shares via instance().
    // Keeping it private forces every code path through create()/instance().
    static SettingsManager *create(QQmlEngine *, QJSEngine *);
    static SettingsManager *instance();

    QString downloadDir() const { return m_downloadDir; }
    void setDownloadDir(const QString &v);

    bool blurNsfw() const { return m_blurNsfw; }
    void setBlurNsfw(bool v);

    bool hideNsfwSites() const { return m_hideNsfwSites; }
    void setHideNsfwSites(bool v);

    QString theme() const { return m_theme; }
    void setTheme(const QString &v);

    QString accentColor() const { return m_accentColor; }
    void setAccentColor(const QString &v);

    QString backgroundTheme() const { return m_backgroundTheme; }
    void setBackgroundTheme(const QString &v);

    QString catalogViewMode() const { return m_catalogViewMode; }
    void setCatalogViewMode(const QString &v);

    bool muteWebmsByDefault() const { return m_muteWebmsByDefault; }
    void setMuteWebmsByDefault(bool v);

    bool birthdayHats() const { return m_birthdayHats; }
    void setBirthdayHats(bool v);

    bool hasCompletedOnboarding() const { return m_hasCompletedOnboarding; }
    void setHasCompletedOnboarding(bool v);

    QVariantList customSites() const { return m_customSites; }
    void setCustomSites(const QVariantList &v);

    bool hydrated() const { return m_hydrated; }

    Q_INVOKABLE void hydrate();
    Q_INVOKABLE void save() const;
    Q_INVOKABLE void setDownloadDirFromUrl(const QUrl &url);

signals:
    void downloadDirChanged();
    void blurNsfwChanged();
    void hideNsfwSitesChanged();
    void themeChanged();
    void accentColorChanged();
    void backgroundThemeChanged();
    void catalogViewModeChanged();
    void muteWebmsByDefaultChanged();
    void birthdayHatsChanged();
    void hasCompletedOnboardingChanged();
    void customSitesChanged();
    void hydratedChanged();

private:
    explicit SettingsManager(QObject *parent = nullptr);

    QString storePath() const;

    QString m_downloadDir;
    bool m_blurNsfw = true;
    bool m_hideNsfwSites = false;
    QString m_theme = QStringLiteral("dark");
    QString m_accentColor = QStringLiteral("#6ee7c9");
    QString m_backgroundTheme = QStringLiteral("none");
    QString m_catalogViewMode = QStringLiteral("grid");
    bool m_muteWebmsByDefault = true;
    bool m_birthdayHats = true;
    bool m_hasCompletedOnboarding = false;
    QVariantList m_customSites;
    bool m_hydrated = false;
};
