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
    //
    // Moves the window manually (tracking mouse deltas) instead of
    // calling startSystemMove(): that enters a native Win32 modal
    // move-loop which desyncs from Qt Quick's threaded render loop and
    // makes the whole window stutter while being dragged. Tracking the
    // drag ourselves stays inside Qt's normal event loop, so dragging
    // can be smooth AND the app can keep the default threaded render
    // loop (better overall animation smoothness - see main.cpp) instead
    // of forcing the slower basic loop just to work around that.
    //
    // Two things were still causing glitching on longer/faster drags
    // even after an earlier Qt.callLater-based throttle:
    //
    // 1. Qt.callLater only coalesces calls that land within the same
    //    event-loop tick. Under sustained fast mouse movement, native
    //    move events can arrive spread across many ticks rather than
    //    bursts within one, so that throttle wasn't actually capping
    //    the update rate the way a real frame budget would. Replaced
    //    with a fixed ~60Hz Timer: onPositionChanged only ever updates
    //    a *target* position (cheap), and the Timer is what actually
    //    calls QWindow::setX/setY (a native SetWindowPos on Windows),
    //    at most once per tick no matter how fast the mouse reports.
    //
    // 2. AnimatedBackground keeps animating continuously underneath
    //    everything (aurora/particles/grid), and moving the window is
    //    already asking the render thread + DWM to do extra work every
    //    frame - the longer the drag, the more accumulated frames were
    //    competing for the same GPU time as those animations. Toggling
    //    WindowDragState.active pauses them for the duration of the
    //    drag (see AnimatedBackground.qml).
    MouseArea {
        id: dragArea
        anchors.fill: parent
        acceptedButtons: Qt.LeftButton
        property point pressGlobal: Qt.point(0, 0)
        property point windowStartPos: Qt.point(0, 0)
        property real pendingX: 0
        property real pendingY: 0

        function endDrag() {
            moveTimer.stop()
            WindowDragState.active = false
        }

        onPressed: (mouse) => {
            pressGlobal = mapToGlobal(mouse.x, mouse.y)
            windowStartPos = Qt.point(root.targetWindow.x, root.targetWindow.y)
            pendingX = windowStartPos.x
            pendingY = windowStartPos.y
            WindowDragState.active = true
            moveTimer.start()
        }
        onPositionChanged: (mouse) => {
            if (pressed && root.targetWindow.visibility !== Window.Maximized) {
                const g = mapToGlobal(mouse.x, mouse.y)
                pendingX = windowStartPos.x + (g.x - pressGlobal.x)
                pendingY = windowStartPos.y + (g.y - pressGlobal.y)
            }
        }
        onReleased: {
            endDrag()
            root.targetWindow.x = pendingX
            root.targetWindow.y = pendingY
        }
        onCanceled: endDrag()
        onDoubleClicked: {
            endDrag()
            root.targetWindow.visibility === Window.Maximized
                ? root.targetWindow.showNormal() : root.targetWindow.showMaximized()
        }

        Timer {
            id: moveTimer
            interval: 16
            repeat: true
            onTriggered: {
                root.targetWindow.x = dragArea.pendingX
                root.targetWindow.y = dragArea.pendingY
            }
        }
    }

    RowLayout {
        anchors.fill: parent
        anchors.leftMargin: root.isMac ? 12 : 16
        anchors.rightMargin: 12
        spacing: 12

        WindowControls { targetWindow: root.targetWindow; visible: root.isMac }

        AppButton {
            flat: true
            iconName: "chevron-left"
            enabled: NavigationController.canGoBack
            opacity: enabled ? 1 : 0.35
            Behavior on opacity { NumberAnimation { duration: 120 } }
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
