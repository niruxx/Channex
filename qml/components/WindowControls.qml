import QtQuick
import QtQuick.Window
import QtQuick.Controls.Basic
import Channex

// Mirrors src/components/layout/WindowControls.tsx: macOS gets
// traffic-light circles on the left of the titlebar, Windows/Linux get
// rectangular minimize/maximize/close buttons on the right.
Row {
    id: root
    required property var targetWindow
    readonly property bool isMac: Qt.platform.os === "osx"
    spacing: isMac ? 8 : 0

    Loader {
        sourceComponent: root.isMac ? macControls : winControls
    }

    Component {
        id: macControls
        Row {
            spacing: 8
            Repeater {
                model: [
                    { color: "#ff5f57", action: "close" },
                    { color: "#febc2e", action: "minimize" },
                    { color: "#28c840", action: "maximize" },
                ]
                delegate: Rectangle {
                    width: 12; height: 12; radius: 6
                    color: modelData.color
                    MouseArea {
                        anchors.fill: parent
                        cursorShape: Qt.PointingHandCursor
                        onClicked: {
                            if (modelData.action === "close") root.targetWindow.close()
                            else if (modelData.action === "minimize") root.targetWindow.showMinimized()
                            else root.targetWindow.visibility === Window.Maximized
                                ? root.targetWindow.showNormal() : root.targetWindow.showMaximized()
                        }
                    }
                }
            }
        }
    }

    // M3 standard icon buttons; close turns error-colored on hover.
    Component {
        id: winControls
        Row {
            spacing: 4
            Repeater {
                model: [
                    { icon: "minus", action: "minimize", danger: false },
                    { icon: root.targetWindow.visibility === Window.Maximized ? "copy" : "square", action: "maximize", danger: false },
                    { icon: "x", action: "close", danger: true },
                ]
                delegate: Rectangle {
                    width: 40; height: 40; radius: 20
                    color: hover.hovered && modelData.danger ? Theme.error : "transparent"
                    Behavior on color { ColorAnimation { duration: Theme.durShort } }

                    StateLayer {
                        visible: !modelData.danger
                        layerColor: Theme.onSurfaceVariantColor
                        hovered: hover.hovered
                        pressed: mouse.pressed
                    }

                    AppIcon {
                        anchors.centerIn: parent
                        name: modelData.icon
                        iconSize: modelData.action === "close" ? 16 : 13
                        color: hover.hovered && modelData.danger ? Theme.onErrorColor : Theme.onSurfaceVariantColor
                    }
                    HoverHandler { id: hover }
                    MouseArea {
                        id: mouse
                        anchors.fill: parent
                        onClicked: {
                            if (modelData.action === "close") root.targetWindow.close()
                            else if (modelData.action === "minimize") root.targetWindow.showMinimized()
                            else root.targetWindow.visibility === Window.Maximized
                                ? root.targetWindow.showNormal() : root.targetWindow.showMaximized()
                        }
                    }
                }
            }
        }
    }
}
