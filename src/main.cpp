#include <QGuiApplication>
#include <QQmlApplicationEngine>
#include <QQmlContext>
#include <QIcon>
#include <QFont>
#include <QFontDatabase>
#include <QFontInfo>

#include "settings/SettingsManager.h"
#include "bookmarks/BookmarksController.h"
#include "lightbox/LightboxController.h"
#include "lightbox/LightboxImageProvider.h"
#include "posting/LynxchanPoster.h"

int main(int argc, char *argv[])
{
    // Deliberately NOT forcing QSG_RENDER_LOOP=basic here: that was an
    // earlier fix for window-drag stutter caused by
    // QQuickWindow::startSystemMove() entering a native Win32 modal
    // move-loop that desyncs from the default threaded render loop.
    // TitleBar.qml now drags the window manually (tracking mouse
    // deltas) instead of calling startSystemMove(), which avoids that
    // modal loop entirely - so the app keeps the default threaded loop,
    // which is materially smoother for everything else (hover/press
    // animations were visibly glitching under the basic loop while any
    // continuous animation, like the animated background, was active).

    QGuiApplication::setOrganizationName(QStringLiteral("Channex"));
    QGuiApplication::setApplicationName(QStringLiteral("Channex Qt"));

    QGuiApplication app(argc, argv);
    app.setWindowIcon(QIcon(QStringLiteral(":/qt/qml/Channex/resources/icons/app.svg")));

    // Google Sans itself is Google's proprietary font and can't be
    // bundled/redistributed here; "Segoe UI Variable" is Windows 11's
    // own modern system UI face (falls back to "Segoe UI" pre-Win11)
    // and reads similarly clean/geometric. Neither exists outside
    // Windows, so forcing the family there would just hand Linux/macOS
    // whatever last-resort font substitution their fontconfig/CoreText
    // happens to pick - instead leave the platform's own default UI
    // font (Cantarell/Noto Sans, SF Pro, ...) alone and only bump size.
#if defined(Q_OS_WIN)
    {
        QFont uiFont(QStringLiteral("Segoe UI Variable Display"));
        if (!QFontInfo(uiFont).exactMatch())
            uiFont.setFamily(QStringLiteral("Segoe UI"));
        uiFont.setPointSize(10);
        QGuiApplication::setFont(uiFont);
    }
#else
    {
        QFont uiFont = QGuiApplication::font();
        uiFont.setPointSize(10);
        QGuiApplication::setFont(uiFont);
    }
#endif

    QQmlApplicationEngine engine;

    // Image providers must exist before any QML tries to load
    // image://lightbox/... or image://captcha/... URLs.
    auto *lightboxProvider = new LightboxImageProvider();
    auto *captchaProvider = new LightboxImageProvider();
    LightboxController::setImageProvider(lightboxProvider);
    setCaptchaImageProvider(captchaProvider);
    engine.addImageProvider(QStringLiteral("lightbox"), lightboxProvider);
    engine.addImageProvider(QStringLiteral("captcha"), captchaProvider);

    // Settings/bookmarks are hydrated once here (not lazily from QML)
    // so main.qml can just check SettingsManager.hydrated on startup.
    SettingsManager::instance()->hydrate();
    BookmarksController::instance()->hydrate();

    QObject::connect(
        &engine, &QQmlApplicationEngine::objectCreationFailed,
        &app, []() { QCoreApplication::exit(-1); },
        Qt::QueuedConnection);

    QObject::connect(&engine, &QQmlApplicationEngine::warnings, &app, [](const QList<QQmlError> &warnings) {
        for (const auto &w : warnings)
            fprintf(stderr, "QML WARNING: %s\n", qPrintable(w.toString()));
    });

    engine.loadFromModule("Channex", "Main");

    return app.exec();
}
