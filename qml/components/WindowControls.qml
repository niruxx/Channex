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

    Component {
        id: winControls
        Row {
            spacing: 0
            Repeater {
                model: [
                    { glyph: "−", action: "minimize", danger: false },
                    { glyph: root.targetWindow.visibility === Window.Maximized ? "⧉" : "□", action: "maximize", danger: false },
                    { glyph: "✕", action: "close", danger: true },
                ]
                delegate: Rectangle {
                    width: 46; height: 32
                    color: hover.hovered ? (modelData.danger ? Theme.danger : Theme.surface3) : "transparent"
                    Text {
                        anchors.centerIn: parent
                        text: modelData.glyph
                        color: hover.hovered && modelData.danger ? "white" : Theme.ink
                        font.pixelSize: 13
                    }
                    HoverHandler { id: hover }
                    MouseArea {
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
