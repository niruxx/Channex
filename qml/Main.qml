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
    color: Theme.canvas

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

    AnimatedBackground {
        anchors.fill: parent
    }

    // Follows the actual window shape: main.cpp clips the frameless
    // window to a rounded region on Windows (SetWindowRgn - DWM's own
    // corner rounding doesn't apply to a borderless popup window), so
    // this outline is rounded to the same radius rather than drawing a
    // square that would get cut off at the true (rounded) corners.
    Rectangle {
        anchors.fill: parent
        radius: 14
        color: "transparent"
        border.width: 1
        border.color: Theme.border
        visible: mainWindow.visibility !== Window.Maximized
        z: 2000
    }

    Loader {
        anchors.fill: parent
        active: !SettingsManager.hydrated
        sourceComponent: Rectangle { color: Theme.canvas }
    }

    Item {
        anchors.fill: parent
        opacity: SettingsManager.hydrated && !SettingsManager.hasCompletedOnboarding ? 1 : 0
        visible: opacity > 0
        Behavior on opacity { NumberAnimation { duration: 200; easing.type: Easing.OutCubic } }

        AnimatedBackground { anchors.fill: parent }
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

                Sidebar { height: parent.height }

                Item {
                    width: parent.width - 240
                    height: parent.height

                    CatalogView {
                        anchors.fill: parent
                        readonly property bool active: NavigationController.view === "catalog"
                        opacity: active ? 1 : 0
                        scale: active ? 1 : 0.985
                        visible: opacity > 0
                        Behavior on opacity { NumberAnimation { duration: 180; easing.type: Easing.OutCubic } }
                        Behavior on scale { NumberAnimation { duration: 180; easing.type: Easing.OutCubic } }
                    }
                    ThreadView {
                        anchors.fill: parent
                        readonly property bool active: NavigationController.view === "thread"
                        opacity: active ? 1 : 0
                        scale: active ? 1 : 0.985
                        visible: opacity > 0
                        Behavior on opacity { NumberAnimation { duration: 180; easing.type: Easing.OutCubic } }
                        Behavior on scale { NumberAnimation { duration: 180; easing.type: Easing.OutCubic } }
                    }
                    BookmarksView {
                        anchors.fill: parent
                        readonly property bool active: NavigationController.view === "bookmarks"
                        opacity: active ? 1 : 0
                        scale: active ? 1 : 0.985
                        visible: opacity > 0
                        Behavior on opacity { NumberAnimation { duration: 180; easing.type: Easing.OutCubic } }
                        Behavior on scale { NumberAnimation { duration: 180; easing.type: Easing.OutCubic } }
                    }
                    DownloadsView {
                        anchors.fill: parent
                        readonly property bool active: NavigationController.view === "downloads"
                        opacity: active ? 1 : 0
                        scale: active ? 1 : 0.985
                        visible: opacity > 0
                        Behavior on opacity { NumberAnimation { duration: 180; easing.type: Easing.OutCubic } }
                        Behavior on scale { NumberAnimation { duration: 180; easing.type: Easing.OutCubic } }
                    }
                    SettingsView {
                        anchors.fill: parent
                        readonly property bool active: NavigationController.view === "settings"
                        opacity: active ? 1 : 0
                        scale: active ? 1 : 0.985
                        visible: opacity > 0
                        Behavior on opacity { NumberAnimation { duration: 180; easing.type: Easing.OutCubic } }
                        Behavior on scale { NumberAnimation { duration: 180; easing.type: Easing.OutCubic } }
                    }
                }
            }
        }

        MediaLightbox { anchors.fill: parent }
    }

    WindowResizeHandles { targetWindow: mainWindow }
}
