#include "IconProvider.h"

#include <QFile>

IconProvider *IconProvider::create(QQmlEngine *, QJSEngine *)
{
    return new IconProvider();
}

IconProvider::IconProvider(QObject *parent)
    : QObject(parent)
{
}

QString IconProvider::rawSvg(const QString &name)
{
    if (name.isEmpty())
        return {};

    const auto cached = m_cache.constFind(name);
    if (cached != m_cache.constEnd())
        return cached.value();

    QFile file(QStringLiteral(":/qt/qml/Channex/resources/icons/ui/%1.svg").arg(name));
    QString content;
    if (file.open(QIODevice::ReadOnly | QIODevice::Text))
        content = QString::fromUtf8(file.readAll());

    m_cache.insert(name, content);
    return content;
}
