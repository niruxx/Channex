import QtQuick
import QtQuick.Controls.Basic
import QtQuick.Window
import Channex
import "components"
import "views"

// Top-level window: mirrors src/App.tsx's composition (custom
// decorations:false chrome, AnimatedBackground behind everything,
// TitleBar + Sidebar + routed main view, MediaLightbox mounted at the
// root so it overlays every view).
ApplicationWindow {
    id: mainWindow
    width: 1440
    height: 900
    minimumWidth: 1024
    minimumHeight: 640
    visible: true
    title: "Channex"
    flags: Qt.Window | Qt.FramelessWindowHint
    // Transparent outside Windows, with the rounded background drawn
    // below: a frameless window is otherwise an opaque rectangle, and
    // its square corners showed past the rounded outline (reading as
    // cut-off corners on KDE). Windows instead clips the window itself
    // to a rounded region in main.cpp (SetWindowRgn).
    color: Qt.platform.os === "windows" ? Theme.surfaceContainer : "transparent"

    readonly property bool roundedCorners: visibility !== Window.Maximized && visibility !== Window.FullScreen
    Binding { target: Theme; property: "windowRadius"; value: mainWindow.roundedCorners ? 14 : 0 }

    // Whole-window fade in/out. Window.opacity is a real native
    // compositor-level fade (SetLayeredWindowAttributes on Windows),
    // not just a QML item's opacity, so this covers everything - chrome
    // included - rather than fading in the content while the window
    // frame itself pops in instantly. Driven by one explicit
    // NumberAnimation (not a Behavior) so fade-out can wait for
    // onFinished before actually quitting, instead of racing the app
    // exiting mid-fade.
    opacity: 0
    property bool closeRequested: false

    NumberAnimation {
        id: fadeAnim
        target: mainWindow
        property: "opacity"
        duration: 220
        easing.type: Easing.OutCubic
        onFinished: if (mainWindow.closeRequested) Qt.quit()
    }

    onClosing: (close) => {
        if (closeRequested) return
        close.accepted = false
        closeRequested = true
        fadeAnim.to = 0
        fadeAnim.restart()
    }

    Component.onCompleted: {
        SitesController.currentSiteId = NavigationController.siteId
        SitesController.loadBoards(NavigationController.siteId)
        fadeAnim.to = 1
        fadeAnim.restart()
    }

    Rectangle {
        anchors.fill: parent
        radius: Theme.windowRadius
        color: Theme.surfaceContainer
    }

    // Inset by 30% of the corner radius - the smallest margin that keeps
    // a square layer fully inside the rounded window shape - so the
    // animated themes can't paint into the transparent corners.
    AnimatedBackground {
        anchors.fill: parent
        anchors.margins: Math.ceil(Theme.windowRadius * 0.3)
    }

    // Follows the actual window shape: main.cpp clips the frameless
    // window to a rounded region on Windows (SetWindowRgn - DWM's own
    // corner rounding doesn't apply to a borderless popup window), so
    // this outline is rounded to the same radius rather than drawing a
    // square that would get cut off at the true (rounded) corners.
    Rectangle {
        anchors.fill: parent
        radius: Theme.windowRadius
        color: "transparent"
        border.width: 1
        border.color: Theme.outlineVariant
        visible: mainWindow.roundedCorners
        z: 2000
    }

    Loader {
        anchors.fill: parent
        active: !SettingsManager.hydrated
        sourceComponent: Rectangle { radius: Theme.windowRadius; color: Theme.surfaceContainer }
    }

    Item {
        anchors.fill: parent
        opacity: SettingsManager.hydrated && !SettingsManager.hasCompletedOnboarding ? 1 : 0
        visible: opacity > 0
        Behavior on opacity { NumberAnimation { duration: 200; easing.type: Easing.OutCubic } }

        AnimatedBackground { anchors.fill: parent; anchors.margins: Math.ceil(Theme.windowRadius * 0.3) }
        OnboardingWizard { anchors.fill: parent }
    }

    Item {
        anchors.fill: parent
        opacity: SettingsManager.hydrated && SettingsManager.hasCompletedOnboarding ? 1 : 0
        visible: opacity > 0
        Behavior on opacity { NumberAnimation { duration: 200; easing.type: Easing.OutCubic } }

        Column {
            anchors.fill: parent
            spacing: 0

            TitleBar { id: titleBar; width: parent.width; targetWindow: mainWindow }

            Row {
                width: parent.width
                height: parent.height - titleBar.height

                Sidebar { id: sidebar; height: parent.height }

                // M3 content pane: the raised `surface` sheet every view
                // renders into, inset from the window's surfaceContainer
                // background on the right/bottom edges.
                Rectangle {
                    width: parent.width - sidebar.width - 12
                    height: parent.height - 12
                    radius: Theme.shapeExtraLarge
                    // Lets an opted-in animated background show through.
                    color: SettingsManager.backgroundTheme === "none" ? Theme.surface : Theme.withAlpha(Theme.surface, 0.82)
                    clip: true

                    CatalogView {
                        width: parent.width
                        height: parent.height
                        readonly property bool active: NavigationController.view === "catalog"
                        opacity: active ? 1 : 0
                        y: active ? 0 : 12
                        visible: opacity > 0
                        Behavior on opacity { NumberAnimation { duration: Theme.durMedium; easing.type: Easing.BezierSpline; easing.bezierCurve: Theme.easeStandard } }
                        Behavior on y { NumberAnimation { duration: Theme.durLong; easing.type: Easing.BezierSpline; easing.bezierCurve: Theme.easeEmphasizedDecel } }
                    }
                    ThreadView {
                        width: parent.width
                        height: parent.height
                        readonly property bool active: NavigationController.view === "thread"
                        opacity: active ? 1 : 0
                        y: active ? 0 : 12
                        visible: opacity > 0
                        Behavior on opacity { NumberAnimation { duration: Theme.durMedium; easing.type: Easing.BezierSpline; easing.bezierCurve: Theme.easeStandard } }
                        Behavior on y { NumberAnimation { duration: Theme.durLong; easing.type: Easing.BezierSpline; easing.bezierCurve: Theme.easeEmphasizedDecel } }
                    }
                    BookmarksView {
                        width: parent.width
                        height: parent.height
                        readonly property bool active: NavigationController.view === "bookmarks"
                        opacity: active ? 1 : 0
                        y: active ? 0 : 12
                        visible: opacity > 0
                        Behavior on opacity { NumberAnimation { duration: Theme.durMedium; easing.type: Easing.BezierSpline; easing.bezierCurve: Theme.easeStandard } }
                        Behavior on y { NumberAnimation { duration: Theme.durLong; easing.type: Easing.BezierSpline; easing.bezierCurve: Theme.easeEmphasizedDecel } }
                    }
                    DownloadsView {
                        width: parent.width
                        height: parent.height
                        readonly property bool active: NavigationController.view === "downloads"
                        opacity: active ? 1 : 0
                        y: active ? 0 : 12
                        visible: opacity > 0
                        Behavior on opacity { NumberAnimation { duration: Theme.durMedium; easing.type: Easing.BezierSpline; easing.bezierCurve: Theme.easeStandard } }
                        Behavior on y { NumberAnimation { duration: Theme.durLong; easing.type: Easing.BezierSpline; easing.bezierCurve: Theme.easeEmphasizedDecel } }
                    }
                    SettingsView {
                        width: parent.width
                        height: parent.height
                        readonly property bool active: NavigationController.view === "settings"
                        opacity: active ? 1 : 0
                        y: active ? 0 : 12
                        visible: opacity > 0
                        Behavior on opacity { NumberAnimation { duration: Theme.durMedium; easing.type: Easing.BezierSpline; easing.bezierCurve: Theme.easeStandard } }
                        Behavior on y { NumberAnimation { duration: Theme.durLong; easing.type: Easing.BezierSpline; easing.bezierCurve: Theme.easeEmphasizedDecel } }
                    }
                }
            }
        }

        MediaLightbox { anchors.fill: parent }
    }

    WindowResizeHandles { targetWindow: mainWindow }
}
