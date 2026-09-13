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

    Component.onCompleted: {
        SitesController.currentSiteId = NavigationController.siteId
        SitesController.loadBoards(NavigationController.siteId)
    }

    AnimatedBackground {
        anchors.fill: parent
    }

    Rectangle {
        anchors.fill: parent
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

            TitleBar { width: parent.width; targetWindow: mainWindow }

            Row {
                width: parent.width
                height: parent.height - 56

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
}
