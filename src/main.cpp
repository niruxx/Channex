#include <QGuiApplication>
#include <QQmlApplicationEngine>
#include <QQmlContext>
#include <QIcon>

#include "settings/SettingsManager.h"
#include "bookmarks/BookmarksController.h"
#include "lightbox/LightboxController.h"
#include "lightbox/LightboxImageProvider.h"
#include "posting/LynxchanPoster.h"

int main(int argc, char *argv[])
{
    QGuiApplication::setOrganizationName(QStringLiteral("Channex"));
    QGuiApplication::setApplicationName(QStringLiteral("Channex Qt"));

    QGuiApplication app(argc, argv);
    app.setWindowIcon(QIcon(QStringLiteral(":/qt/qml/Channex/resources/icons/app.svg")));

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

    fprintf(stderr, "channex_qt: starting up\n");
    fflush(stderr);

    QObject::connect(&engine, &QQmlApplicationEngine::warnings, &app, [](const QList<QQmlError> &warnings) {
        for (const auto &w : warnings)
            fprintf(stderr, "QML WARNING: %s\n", qUtf8Printable(w.toString()));
        fflush(stderr);
    });

    QObject::connect(
        &engine, &QQmlApplicationEngine::objectCreationFailed,
        &app, []() {
            fprintf(stderr, "channex_qt: object creation failed\n");
            fflush(stderr);
            QCoreApplication::exit(-1);
        },
        Qt::QueuedConnection);

    engine.loadFromModule("Channex", "Main");
    fprintf(stderr, "channex_qt: loadFromModule returned\n");
    fflush(stderr);

    return app.exec();
}
