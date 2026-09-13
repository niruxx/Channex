import QtQuick
import QtQuick.Controls.Basic
import QtQuick.Layouts
import Channex

// Mirrors the <Sidebar> composition in src/App.tsx: SiteSwitcher +
// BoardList on top, Bookmarks/Downloads/Settings nav at the bottom.
Rectangle {
    id: root
    width: 240
    color: Theme.surface

    Rectangle { anchors.right: parent.right; width: 1; height: parent.height; color: Theme.borderSoft }

    ColumnLayout {
        anchors.fill: parent
        anchors.margins: 12
        spacing: 12

        SiteSwitcher { Layout.fillWidth: true }

        Rectangle { Layout.fillWidth: true; height: 1; color: Theme.borderSoft }

        BoardList { Layout.fillWidth: true; Layout.fillHeight: true }

        Rectangle { Layout.fillWidth: true; height: 1; color: Theme.borderSoft }

        ColumnLayout {
            Layout.fillWidth: true
            spacing: 2

            Repeater {
                model: [
                    { label: "Bookmarks", view: "bookmarks" },
                    { label: "Downloads", view: "downloads" },
                    { label: "Settings", view: "settings" },
                ]
                delegate: Rectangle {
                    id: navDelegate
                    Layout.fillWidth: true
                    height: 32
                    radius: Theme.radiusSm
                    readonly property bool current: NavigationController.view === modelData.view
                    color: current ? Theme.surface3 : (navHover.hovered ? Theme.surface2 : "transparent")

                    Behavior on color { ColorAnimation { duration: 120; easing.type: Easing.OutCubic } }

                    Text {
                        anchors.verticalCenter: parent.verticalCenter
                        anchors.left: parent.left
                        anchors.leftMargin: 10
                        text: modelData.label
                        color: navDelegate.current ? Theme.accent : Theme.inkDim
                        font.pixelSize: 13

                        Behavior on color { ColorAnimation { duration: 120 } }
                    }

                    HoverHandler { id: navHover }

                    MouseArea {
                        anchors.fill: parent
                        cursorShape: Qt.PointingHandCursor
                        onClicked: {
                            if (modelData.view === "bookmarks") NavigationController.goBookmarks()
                            else if (modelData.view === "downloads") NavigationController.goDownloads()
                            else NavigationController.goSettings()
                        }
                    }
                }
            }
        }
    }
}
