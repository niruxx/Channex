import QtQuick
import QtQuick.Controls.Basic
import QtQuick.Layouts
import QtQuick.Window
import Channex

// Mirrors src/components/layout/TitleBar.tsx: custom-drawn header with
// an OS-drag region, a back button, a breadcrumb trail, and
// platform-ordered window controls (mac: traffic lights on the left;
// win/linux: rectangular controls on the right).
//
// Styled as an M3 small top app bar sitting on the window's
// surfaceContainer background (no divider - the raised content pane
// below provides the separation): app mark + name, a back icon
// button, the current location as a title, window controls. Its top
// corners don't need an explicit radius to match the window's rounded
// shape: main.cpp clips the whole window to a rounded region on
// Windows (SetWindowRgn), and that clip applies to this bar too.
Item {
    id: root
    required property var targetWindow
    height: 60

    readonly property bool isMac: Qt.platform.os === "osx"
    readonly property bool isWindows: Qt.platform.os === "windows"
    readonly property var currentSite: SitesController.findSite(NavigationController.siteId)

    readonly property bool inSiteView: NavigationController.view === "catalog" || NavigationController.view === "thread"
    readonly property string siteName: currentSite.name || NavigationController.siteId

    readonly property string title: {
        switch (NavigationController.view) {
        case "bookmarks": return "Bookmarks"
        case "downloads": return "Downloads"
        case "settings": return "Settings"
        case "thread": return "/" + NavigationController.boardCode + "/ · Thread " + NavigationController.threadId
        default: return NavigationController.boardCode ? "/" + NavigationController.boardCode + "/" : root.siteName
        }
    }
    readonly property string subtitle: inSiteView ? siteName : ""

    // Drag region - sits behind the interactive controls (they're
    // instantiated after this in the RowLayout below, so they win
    // Z-order/hit-testing by default). Fills the whole strip, including
    // the gaps around the floating bar below, so the entire header
    // height stays grabbable rather than just the visible card.
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
    // Several things were causing glitching / sub-native-feeling drags:
    //
    // 1. Qt.callLater only coalesces calls that land within the same
    //    event-loop tick. Under sustained fast mouse movement, native
    //    move events can arrive spread across many ticks rather than
    //    bursts within one, so that throttle wasn't actually capping
    //    the update rate the way a real frame budget would.
    //
    // 2. A follow-up fixed-interval (~60Hz) Timer fixed that, but a
    //    fixed interval can only ever approximate the display's actual
    //    refresh rate - it hard-capped every drag to 60Hz even on a
    //    240Hz screen, which felt slower than native window drags
    //    there. Tried driving position updates off the window's own
    //    frameSwapped signal instead (calling update() from inside the
    //    handler to keep it self-triggering) to tie the rate to real
    //    vsync - that backfired badly: nothing here actually guarantees
    //    that a manually-requested update()/frameSwapped cycle blocks
    //    on the real display refresh the way natural repaints do, so it
    //    could spin far faster and more erratically than the display's
    //    Hz, firing the expensive native SetWindowPos call way more
    //    often (and less predictably) than the old timer ever did -
    //    much worse jank, not better.
    //
    //    Reverted to a Timer, but its interval now tracks the window's
    //    actual screen refresh rate (Screen.refreshRate) instead of a
    //    hardcoded 60Hz guess, so it still runs close to a 240Hz
    //    screen's real cadence without the runaway risk of the
    //    self-driving frameSwapped approach - a bounded, predictable
    //    rate beats an unbounded one that happens to be aimed at vsync.
    //
    // 3. AnimatedBackground keeps animating continuously underneath
    //    everything (aurora/particles/grid), and moving the window is
    //    already asking the render thread + DWM to do extra work every
    //    frame - the longer the drag, the more accumulated frames were
    //    competing for the same GPU time as those animations. Toggling
    //    WindowDragState.active pauses them for the duration of the
    //    drag (see AnimatedBackground.qml).
    //
    // This manual tracking is Windows-only. On X11/macOS it isn't
    // needed (startSystemMove() doesn't hit that Win32 modal-loop
    // desync there), and on Wayland it flat-out can't work: a Wayland
    // client has no ability to set its own global position at all -
    // window.x/window.y writes are silently ignored by the compositor.
    // startSystemMove() is the only thing that moves a frameless window
    // there (it hands off to the compositor's xdg_toplevel move), so
    // every non-Windows platform uses it instead.
    MouseArea {
        id: dragArea
        anchors.fill: parent
        acceptedButtons: Qt.LeftButton
        property point pressGlobal: Qt.point(0, 0)
        property point windowStartPos: Qt.point(0, 0)
        property real pendingX: 0
        property real pendingY: 0

        // Clamped so a bogus/unreported refresh rate (0, -1) can't turn
        // the Timer's interval into 0/negative or something absurdly
        // small; 240 is a generous upper bound for current high-refresh
        // monitors while still being far better than a flat 60.
        readonly property real screenHz: {
            const hz = root.targetWindow.screen ? root.targetWindow.screen.refreshRate : 60
            return (hz > 0 && hz < 1000) ? hz : 60
        }

        function endDrag() {
            moveTimer.stop()
            WindowDragState.active = false
        }

        onPressed: (mouse) => {
            if (!root.isWindows) {
                root.targetWindow.startSystemMove()
                return
            }
            pressGlobal = mapToGlobal(mouse.x, mouse.y)
            windowStartPos = Qt.point(root.targetWindow.x, root.targetWindow.y)
            pendingX = windowStartPos.x
            pendingY = windowStartPos.y
            WindowDragState.active = true
            moveTimer.start()
        }
        onPositionChanged: (mouse) => {
            if (root.isWindows && pressed && root.targetWindow.visibility !== Window.Maximized) {
                const g = mapToGlobal(mouse.x, mouse.y)
                pendingX = windowStartPos.x + (g.x - pressGlobal.x)
                pendingY = windowStartPos.y + (g.y - pressGlobal.y)
            }
        }
        onReleased: {
            if (!root.isWindows) return
            endDrag()
            root.targetWindow.x = pendingX
            root.targetWindow.y = pendingY
        }
        onCanceled: { if (root.isWindows) endDrag() }
        onDoubleClicked: {
            if (root.isWindows) endDrag()
            root.targetWindow.visibility === Window.Maximized
                ? root.targetWindow.showNormal() : root.targetWindow.showMaximized()
        }

        Timer {
            id: moveTimer
            interval: Math.max(1, Math.round(1000 / dragArea.screenHz))
            repeat: true
            onTriggered: {
                root.targetWindow.x = dragArea.pendingX
                root.targetWindow.y = dragArea.pendingY
            }
        }
    }

    RowLayout {
        anchors.fill: parent
        anchors.leftMargin: root.isMac ? 16 : 20
        anchors.rightMargin: 12
        spacing: 8

        WindowControls { targetWindow: root.targetWindow; visible: root.isMac }

        // Brand block - lines up with the navigation drawer below it.
        // Explicit fillWidth: false - a RowLayout child that itself holds
        // a fillWidth item would otherwise inherit fillWidth and grab
        // half the bar.
        RowLayout {
            Layout.fillWidth: false
            Layout.preferredWidth: root.isMac ? 160 : 236
            Layout.maximumWidth: Layout.preferredWidth
            spacing: 10

            Rectangle {
                implicitWidth: 32; implicitHeight: 32; radius: Theme.shapeMedium
                color: Theme.primaryContainer
                Image {
                    anchors.centerIn: parent
                    width: 20; height: 20
                    source: "qrc:/qt/qml/Channex/resources/icons/app.svg"
                    fillMode: Image.PreserveAspectFit
                    smooth: true
                }
            }
            Text {
                Layout.fillWidth: true
                text: "Channex"
                color: Theme.onSurfaceColor
                font.pixelSize: Theme.titleLarge
                font.weight: Font.Medium
                elide: Text.ElideRight
            }
        }

        AppButton {
            variant: "icon"
            iconName: "chevron-left"
            tooltip: "Back"
            enabled: NavigationController.canGoBack
            onClicked: NavigationController.back()
        }

        // Site avatar - matches the selected avatar in the drawer.
        Rectangle {
            visible: root.inSiteView
            implicitWidth: 32; implicitHeight: 32
            radius: Theme.shapeMedium
            color: Theme.primaryContainer
            clip: true

            readonly property bool hasFavicon: !!root.currentSite.favicon && root.currentSite.favicon.length > 0

            Text {
                anchors.centerIn: parent
                visible: !parent.hasFavicon || favicon.status !== Image.Ready
                text: (root.currentSite.name || "?").charAt(0).toUpperCase()
                color: Theme.onPrimaryContainerColor
                font.pixelSize: Theme.titleSmall
                font.weight: Font.Bold
            }

            Image {
                id: favicon
                anchors.centerIn: parent
                width: 18; height: 18
                visible: parent.hasFavicon && status === Image.Ready
                source: parent.hasFavicon ? root.currentSite.favicon : ""
                fillMode: Image.PreserveAspectFit
                asynchronous: true
                cache: true
            }
        }

        ColumnLayout {
            Layout.fillWidth: true
            Layout.leftMargin: 4
            spacing: 0
            Text {
                Layout.fillWidth: true
                text: root.title
                color: Theme.onSurfaceColor
                font.pixelSize: Theme.titleMedium
                font.weight: Font.Medium
                elide: Text.ElideMiddle
            }
            Text {
                Layout.fillWidth: true
                visible: root.subtitle.length > 0
                text: root.subtitle
                color: Theme.onSurfaceVariantColor
                font.pixelSize: Theme.bodySmall
                elide: Text.ElideRight
            }
        }

        WindowControls { targetWindow: root.targetWindow; visible: !root.isMac }
    }
}
