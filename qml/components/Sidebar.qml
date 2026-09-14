import QtQuick
import QtQuick.Controls.Basic
import QtQuick.Layouts
import Channex

// Mirrors the <Sidebar> composition in src/App.tsx: SiteSwitcher +
// BoardList on top, Bookmarks/Downloads/Settings nav at the bottom.
// Rendered as a floating, outlined card inset from the window edges
// (rather than a flat panel flush against them) so it reads as a
// distinct, modern surface against the canvas/animated background
// behind it.
Item {
    id: root
    width: 240

    Rectangle {
        anchors.fill: parent
        anchors.topMargin: 10
        anchors.bottomMargin: 10
        anchors.leftMargin: 10
        anchors.rightMargin: 6
        radius: Theme.radiusLg
        color: Theme.surface
        border.width: 1
        border.color: Theme.border

        CardShadow { anchors.fill: parent; radius: Theme.radiusLg }

        ColumnLayout {
            anchors.fill: parent
            anchors.margins: 16
            spacing: 14

            RowLayout {
                Layout.fillWidth: true
                spacing: 8

                Image {
                    Layout.preferredWidth: 26
                    Layout.preferredHeight: 26
                    source: "qrc:/qt/qml/Channex/resources/icons/app.svg"
                    fillMode: Image.PreserveAspectFit
                    smooth: true
                }
                Text {
                    text: "Channex"
                    color: Theme.ink
                    font.pixelSize: 15
                    font.bold: true
                    Layout.fillWidth: true
                }
            }

            Rectangle { Layout.fillWidth: true; height: 1; color: Theme.borderSoft }

            SiteSwitcher { Layout.fillWidth: true }

            Rectangle { Layout.fillWidth: true; height: 1; color: Theme.borderSoft }

            BoardList { Layout.fillWidth: true; Layout.fillHeight: true }

            Rectangle { Layout.fillWidth: true; height: 1; color: Theme.borderSoft }

            ColumnLayout {
                Layout.fillWidth: true
                spacing: 2

                Repeater {
                    model: [
                        { label: "Bookmarks", view: "bookmarks", icon: "bookmark" },
                        { label: "Downloads", view: "downloads", icon: "download" },
                        { label: "Settings", view: "settings", icon: "gear" },
                    ]
                    delegate: Rectangle {
                        id: navDelegate
                        Layout.fillWidth: true
                        height: 34
                        radius: Theme.radiusSm
                        readonly property bool current: NavigationController.view === modelData.view
                        color: current ? Theme.surface3 : (navHover.hovered ? Theme.surface2 : "transparent")

                        Behavior on color { ColorAnimation { duration: 120; easing.type: Easing.OutCubic } }

                        RowLayout {
                            anchors.fill: parent
                            anchors.leftMargin: 10
                            anchors.rightMargin: 10
                            spacing: 8

                            AppIcon {
                                name: modelData.icon
                                iconSize: 15
                                color: navDelegate.current ? Theme.accent : Theme.inkDim
                                Behavior on color { ColorAnimation { duration: 120 } }
                            }

                            Text {
                                Layout.fillWidth: true
                                text: modelData.label
                                color: navDelegate.current ? Theme.accent : Theme.inkDim
                                font.pixelSize: 13

                                Behavior on color { ColorAnimation { duration: 120 } }
                            }
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
}
