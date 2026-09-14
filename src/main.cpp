#include <QGuiApplication>
#include <QQmlApplicationEngine>
#include <QQmlContext>
#include <QIcon>
#include <QFont>
#include <QFontDatabase>
#include <QFontInfo>
#include <QQuickWindow>

#include "settings/SettingsManager.h"
#include "bookmarks/BookmarksController.h"
#include "lightbox/LightboxController.h"
#include "lightbox/LightboxImageProvider.h"
#include "posting/LynxchanPoster.h"

#if defined(Q_OS_WIN)
#include <windows.h>
#endif

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

    engine.loadFromModule("Channex", "Main");

    // Removing the native frame (Qt::FramelessWindowHint, set on
    // mainWindow in Main.qml) also removes Windows 11's automatic
    // rounded-corner treatment. DWM's auto-rounding (and even the
    // explicit DWMWA_WINDOW_CORNER_PREFERENCE override - tried first;
    // it reports success but has no visible effect here) only applies
    // to windows that still carry the standard WS_CAPTION/WS_THICKFRAME
    // frame styles, which a Qt::FramelessWindowHint popup doesn't have.
    // SetWindowRgn instead clips the window's own paintable region
    // directly - it doesn't care what style the window has, so it
    // reliably rounds a plain popup window's corners. The tradeoff is a
    // hard-edged (non-antialiased) region rather than DWM's smooth
    // compositor rounding, a minor cosmetic cost for a window that
    // couldn't be rounded at all otherwise. Skipped while maximized/
    // fullscreen, matching how every other maximized window looks.
#if defined(Q_OS_WIN)
    if (!engine.rootObjects().isEmpty()) {
        if (auto *window = qobject_cast<QQuickWindow *>(engine.rootObjects().constFirst())) {
            auto applyRoundedRegion = [window]() {
                const auto hwnd = reinterpret_cast<HWND>(window->winId());
                if (window->visibility() == QWindow::Maximized || window->visibility() == QWindow::FullScreen) {
                    SetWindowRgn(hwnd, nullptr, TRUE);
                    return;
                }
                RECT rect{};
                if (!GetClientRect(hwnd, &rect)) return;
                const int w = rect.right - rect.left;
                const int h = rect.bottom - rect.top;
                if (w <= 0 || h <= 0) return;
                const int radius = static_cast<int>(14 * window->devicePixelRatio() + 0.5);
                HRGN region = CreateRoundRectRgn(0, 0, w + 1, h + 1, radius, radius);
                if (region && !SetWindowRgn(hwnd, region, TRUE))
                    DeleteObject(region);
            };
            applyRoundedRegion();
            QObject::connect(window, &QQuickWindow::widthChanged, window, applyRoundedRegion);
            QObject::connect(window, &QQuickWindow::heightChanged, window, applyRoundedRegion);
            QObject::connect(window, &QQuickWindow::visibilityChanged, window, applyRoundedRegion);
        }
    }
#endif

    return app.exec();
}
