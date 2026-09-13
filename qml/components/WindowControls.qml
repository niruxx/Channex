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

    // Modern, non-blocky treatment: round hover pills with gaps between
    // them, instead of the old flush rectangular hit-boxes that read as
    // one solid square bar.
    Component {
        id: winControls
        Row {
            spacing: 6
            Repeater {
                model: [
                    { icon: "minus", action: "minimize", danger: false },
                    { icon: root.targetWindow.visibility === Window.Maximized ? "copy" : "square", action: "maximize", danger: false },
                    { icon: "x", action: "close", danger: true },
                ]
                delegate: Rectangle {
                    width: 32; height: 32; radius: 16
                    color: hover.hovered ? (modelData.danger ? Theme.danger : Theme.surface3) : "transparent"
                    scale: mouse.pressed ? 0.88 : 1.0
                    Behavior on color { ColorAnimation { duration: 100 } }
                    Behavior on scale { NumberAnimation { duration: 90; easing.type: Easing.OutCubic } }

                    AppIcon {
                        anchors.centerIn: parent
                        name: modelData.icon
                        iconSize: 11
                        color: hover.hovered && modelData.danger ? "white" : Theme.ink
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
