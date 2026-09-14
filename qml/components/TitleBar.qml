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
// Drawn as a floating, fully-rounded card inset from the window edges -
// the same "surface floating over the animated background" language
// Sidebar.qml uses - rather than a flush square header, with a
// pill-shaped breadcrumb and a circular, accent-ringed site avatar for
// a more modern feel.
Item {
    id: root
    required property var targetWindow
    height: 68

    readonly property bool isMac: Qt.platform.os === "osx"
    readonly property bool isWindows: Qt.platform.os === "windows"
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

    // The actual visible titlebar surface - a floating rounded card
    // inset from the window edges, matching Sidebar's card treatment.
    Rectangle {
        id: bar
        anchors.fill: parent
        anchors.margins: 8
        radius: Theme.radiusLg
        color: Theme.surface
        border.width: 1
        border.color: Theme.border
        clip: true

        // Subtle top-to-mid sheen for a bit of glassy depth. A no-op in
        // light theme (Theme.surface is already white, so lightening it
        // clamps back to white), a soft highlight in dark theme.
        Rectangle {
            anchors.fill: parent
            radius: parent.radius
            opacity: 0.5
            gradient: Gradient {
                GradientStop { position: 0.0; color: Qt.lighter(Theme.surface, 1.35) }
                GradientStop { position: 0.6; color: Theme.surface }
            }
        }

        // Thin accent glow along the top edge - a small techy signature
        // touch tying the bar back to the user's chosen accent color.
        Rectangle {
            anchors.top: parent.top
            anchors.left: parent.left
            anchors.right: parent.right
            anchors.margins: parent.radius * 0.5
            height: 2
            radius: 1
            opacity: 0.55
            gradient: Gradient {
                orientation: Gradient.Horizontal
                GradientStop { position: 0.0; color: "transparent" }
                GradientStop { position: 0.5; color: Theme.accent }
                GradientStop { position: 1.0; color: "transparent" }
            }
        }

        RowLayout {
            anchors.fill: parent
            anchors.leftMargin: root.isMac ? 14 : 16
            anchors.rightMargin: 14
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

            // Circular site avatar with a soft accent ring, in place of
            // the old flat square badge.
            Item {
                width: 30; height: 30
                visible: NavigationController.view !== "bookmarks" && NavigationController.view !== "downloads" && NavigationController.view !== "settings"

                readonly property bool hasFavicon: !!root.currentSite.favicon && root.currentSite.favicon.length > 0

                Rectangle {
                    anchors.centerIn: parent
                    width: 30; height: 30; radius: 15
                    color: "transparent"
                    border.width: 2
                    border.color: root.currentSite.accent || Theme.accent
                    opacity: 0.35
                }

                Rectangle {
                    anchors.centerIn: parent
                    width: 24; height: 24; radius: 12
                    color: root.currentSite.accent || Theme.accent
                    clip: true

                    Text {
                        anchors.centerIn: parent
                        visible: !parent.parent.hasFavicon || favicon.status !== Image.Ready
                        text: (root.currentSite.name || "?").charAt(0).toUpperCase()
                        color: "#0c0e11"
                        font.pixelSize: 11
                        font.bold: true
                    }

                    Image {
                        id: favicon
                        anchors.centerIn: parent
                        width: 16; height: 16
                        visible: parent.parent.hasFavicon && status === Image.Ready
                        source: parent.parent.hasFavicon ? root.currentSite.favicon : ""
                        fillMode: Image.PreserveAspectFit
                        asynchronous: true
                        cache: true
                    }
                }
            }

            // Breadcrumb as a pill-shaped "chip" rather than bare text.
            Rectangle {
                Layout.fillWidth: true
                Layout.preferredHeight: 32
                radius: height / 2
                color: Theme.surface2
                border.width: 1
                border.color: Theme.borderSoft

                Text {
                    anchors.fill: parent
                    anchors.leftMargin: 14
                    anchors.rightMargin: 14
                    verticalAlignment: Text.AlignVCenter
                    text: root.breadcrumb
                    color: Theme.ink
                    font.pixelSize: 13
                    elide: Text.ElideMiddle
                }
            }

            WindowControls { targetWindow: root.targetWindow; visible: !root.isMac }
        }
    }
}
