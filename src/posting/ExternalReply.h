#pragma once

#include <QObject>
#include <QQmlEngine>
#include <QString>

// Mirrors src/lib/externalReply.ts, with a deliberate scope reduction:
// the Tauri app opens an embedded secondary WebviewWindow and
// auto-refreshes the thread when the user closes it. A faithful port
// of that needs QtWebEngine embedded in a popup window; until that's
// built, this opens the system's default browser (QDesktopServices)
// and copies the composed comment to the clipboard so the user can
// paste it into the site's own reply form, same as upstream's
// clipboard step. There is no close-detection, so the thread does not
// auto-refresh - the user has to hit refresh manually. Tracked as a
// known parity gap, see QT/README.md.
class ExternalReply : public QObject
{
    Q_OBJECT
    QML_ELEMENT
    QML_SINGLETON

public:
    // Constructor stays private: a public one makes this look default-
    // constructible to QML_SINGLETON's factory detection, which can then
    // bypass create() and hand QML a disconnected instance.
    static ExternalReply *create(QQmlEngine *, QJSEngine *);

    Q_INVOKABLE void openForReply(const QString &threadUrl, const QString &commentToCopy);
    Q_INVOKABLE void openAuthPage(const QString &url);
    Q_INVOKABLE void openUrl(const QString &url);

private:
    explicit ExternalReply(QObject *parent = nullptr);
};
