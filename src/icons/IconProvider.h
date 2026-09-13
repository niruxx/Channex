#pragma once

#include <QHash>
#include <QObject>
#include <QQmlEngine>
#include <QString>

// Reads (and caches) the raw text of resources/icons/ui/<name>.svg for
// AppIcon.qml to recolor and turn into a data: URL. Done in C++ rather
// than via QML's XMLHttpRequest because Qt disables XHR reads of local
// files/qrc resources by default (QML_XHR_ALLOW_FILE_READ) - fine for a
// one-off dev workaround, not something to rely on in a shipped app.
class IconProvider : public QObject
{
    Q_OBJECT
    QML_ELEMENT
    QML_SINGLETON

public:
    static IconProvider *create(QQmlEngine *, QJSEngine *);

    explicit IconProvider(QObject *parent = nullptr);

    Q_INVOKABLE QString rawSvg(const QString &name);

private:
    QHash<QString, QString> m_cache;
};
