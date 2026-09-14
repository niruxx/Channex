import QtQuick
import QtQuick.Window
import Channex

// Thin, invisible edge/corner strips that call Window.startSystemResize()
// - the standard cross-platform way to hand an interactive resize off to
// the OS/compositor, the same mechanism TitleBar.qml already uses for
// startSystemMove() on non-Windows platforms. Qt::FramelessWindowHint
// (set on mainWindow in Main.qml) strips the native resize border
// entirely, so without this the window could only ever change size
// programmatically (minimumWidth/Height) - never by a user dragging an
// edge, which is the resizing capability this app was missing.
Item {
    id: root
    required property var targetWindow
    anchors.fill: parent
    z: 3000
    enabled: root.targetWindow.visibility !== Window.Maximized

    readonly property int edgeThickness: 6
    readonly property int cornerSize: 14

    MouseArea {
        anchors.left: parent.left
        anchors.right: parent.right
        anchors.top: parent.top
        height: root.edgeThickness
        cursorShape: Qt.SizeVerCursor
        onPressed: root.targetWindow.startSystemResize(Qt.TopEdge)
    }
    MouseArea {
        anchors.left: parent.left
        anchors.right: parent.right
        anchors.bottom: parent.bottom
        height: root.edgeThickness
        cursorShape: Qt.SizeVerCursor
        onPressed: root.targetWindow.startSystemResize(Qt.BottomEdge)
    }
    MouseArea {
        anchors.top: parent.top
        anchors.bottom: parent.bottom
        anchors.left: parent.left
        width: root.edgeThickness
        cursorShape: Qt.SizeHorCursor
        onPressed: root.targetWindow.startSystemResize(Qt.LeftEdge)
    }
    MouseArea {
        anchors.top: parent.top
        anchors.bottom: parent.bottom
        anchors.right: parent.right
        width: root.edgeThickness
        cursorShape: Qt.SizeHorCursor
        onPressed: root.targetWindow.startSystemResize(Qt.RightEdge)
    }

    // Corners declared after the edges so they win the overlap.
    MouseArea {
        anchors.left: parent.left
        anchors.top: parent.top
        width: root.cornerSize
        height: root.cornerSize
        cursorShape: Qt.SizeFDiagCursor
        onPressed: root.targetWindow.startSystemResize(Qt.TopEdge | Qt.LeftEdge)
    }
    MouseArea {
        anchors.right: parent.right
        anchors.top: parent.top
        width: root.cornerSize
        height: root.cornerSize
        cursorShape: Qt.SizeBDiagCursor
        onPressed: root.targetWindow.startSystemResize(Qt.TopEdge | Qt.RightEdge)
    }
    MouseArea {
        anchors.left: parent.left
        anchors.bottom: parent.bottom
        width: root.cornerSize
        height: root.cornerSize
        cursorShape: Qt.SizeBDiagCursor
        onPressed: root.targetWindow.startSystemResize(Qt.BottomEdge | Qt.LeftEdge)
    }
    MouseArea {
        anchors.right: parent.right
        anchors.bottom: parent.bottom
        width: root.cornerSize
        height: root.cornerSize
        cursorShape: Qt.SizeFDiagCursor
        onPressed: root.targetWindow.startSystemResize(Qt.BottomEdge | Qt.RightEdge)
    }
}
