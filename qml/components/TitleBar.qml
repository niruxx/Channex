import QtQuick
import QtQuick.Controls.Basic
import QtQuick.Layouts
import QtQuick.Window
import Channex

// Mirrors src/components/layout/TitleBar.tsx: 56px custom-drawn header
// with an OS-drag region, a back button, a breadcrumb trail, and
// platform-ordered window controls (mac: traffic lights on the left;
// win/linux: rectangular controls on the right).
Rectangle {
    id: root
    required property var targetWindow
    height: 56
    color: Theme.surface

    readonly property bool isMac: Qt.platform.os === "osx"
    readonly property var currentSite: SitesController.findSite(NavigationController.siteId)

    readonly property string breadcrumb: {
        switch (NavigationController.view) {
        case "bookmarks": return "Bookmarks"
        case "downloads": return "Downloads"
        case "settings": return "Settings"
        case "thread": return (currentSite.name || NavigationController.siteId) + " / " + NavigationController.boardCode + " / Thread #" + NavigationController.threadId
        default: return (currentSite.name || NavigationController.siteId) + (NavigationController.boardCode ? " / " + NavigationController.boardCode : "")
        }
    }

    Rectangle { anchors.bottom: parent.bottom; width: parent.width; height: 1; color: Theme.borderSoft }

    // Drag region - sits behind the interactive controls (they're
    // instantiated after this in the RowLayout below, so they win
    // Z-order/hit-testing by default).
    MouseArea {
        anchors.fill: parent
        acceptedButtons: Qt.LeftButton
        onPressed: root.targetWindow.startSystemMove()
        onDoubleClicked: root.targetWindow.visibility === Window.Maximized
            ? root.targetWindow.showNormal() : root.targetWindow.showMaximized()
    }

    RowLayout {
        anchors.fill: parent
        anchors.leftMargin: root.isMac ? 12 : 16
        anchors.rightMargin: 12
        spacing: 12

        WindowControls { targetWindow: root.targetWindow; visible: root.isMac }

        AppButton {
            flat: true
            enabled: NavigationController.canGoBack
            opacity: enabled ? 1 : 0.35
            text: "‹"
            font.pixelSize: 18
            onClicked: NavigationController.back()
        }

        Rectangle {
            width: 22; height: 22; radius: 6
            color: root.currentSite.accent || Theme.accent
            visible: NavigationController.view !== "bookmarks" && NavigationController.view !== "downloads" && NavigationController.view !== "settings"
            clip: true

            readonly property bool hasFavicon: !!root.currentSite.favicon && root.currentSite.favicon.length > 0

            Text {
                anchors.centerIn: parent
                visible: !parent.hasFavicon || favicon.status !== Image.Ready
                text: (root.currentSite.name || "?").charAt(0).toUpperCase()
                color: "#0c0e11"
                font.pixelSize: 11
                font.bold: true
            }

            Image {
                id: favicon
                anchors.centerIn: parent
                width: 16; height: 16
                visible: parent.hasFavicon && status === Image.Ready
                source: parent.hasFavicon ? root.currentSite.favicon : ""
                fillMode: Image.PreserveAspectFit
                asynchronous: true
                cache: true
            }
        }

        Text {
            text: root.breadcrumb
            color: Theme.ink
            font.pixelSize: 13
            elide: Text.ElideMiddle
            Layout.fillWidth: true
        }

        WindowControls { targetWindow: root.targetWindow; visible: !root.isMac }
    }
}
