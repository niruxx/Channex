import QtQuick
import QtQuick.Particles
import Channex

// Approximates src/components/layout/AnimatedBackground.tsx's three
// CSS-only themes (aurora / particles / grid) using QML-native
// mechanisms instead of a 1:1 CSS keyframe port: drifting blurred
// blobs, a QtQuick.Particles emitter, and a panning Canvas grid.
Item {
    id: root
    anchors.fill: parent
    z: -1

    // Flat by design: no gradient wash here - with backgroundTheme
    // "none" (the default) the window's own flat Theme.canvas fill
    // (see Main.qml) is the entire background, exactly as a flat UI
    // calls for. The opt-in aurora/particles/grid themes below are the
    // only source of any motion or depth, unchanged.

    // ---- aurora ----
    Item {
        anchors.fill: parent
        visible: SettingsManager.backgroundTheme === "aurora"
        opacity: Theme.dark ? 0.32 : 0.22

        Repeater {
            model: 3
            delegate: Rectangle {
                id: blob
                property real baseX: [0.1, 0.6, 0.3][index] * root.width
                property real baseY: [0.15, 0.2, 0.7][index] * root.height
                width: root.width * 0.55
                height: width
                radius: width / 2
                color: index === 1 ? Theme.accent2 : Theme.accent
                x: baseX
                y: baseY

                // Paused while the window is being dragged (see
                // TitleBar.qml / WindowDragState) - one less thing
                // competing with the render thread for GPU time while
                // the window itself is also being repositioned every
                // frame.
                SequentialAnimation on x {
                    running: !WindowDragState.active
                    loops: Animation.Infinite
                    NumberAnimation { to: blob.baseX + 80; duration: 9000 + index * 4000; easing.type: Easing.InOutSine }
                    NumberAnimation { to: blob.baseX - 60; duration: 9000 + index * 4000; easing.type: Easing.InOutSine }
                }
                SequentialAnimation on y {
                    running: !WindowDragState.active
                    loops: Animation.Infinite
                    NumberAnimation { to: blob.baseY - 60; duration: 11000 + index * 3000; easing.type: Easing.InOutSine }
                    NumberAnimation { to: blob.baseY + 80; duration: 11000 + index * 3000; easing.type: Easing.InOutSine }
                }
            }
        }
    }

    // ---- particles ----
    ParticleSystem {
        id: particleSystem
        anchors.fill: parent
        running: SettingsManager.backgroundTheme === "particles" && !WindowDragState.active
        visible: SettingsManager.backgroundTheme === "particles"

        ItemParticle {
            delegate: Rectangle {
                width: 4; height: 4; radius: 2
                color: Theme.accent
                opacity: 0.6
            }
        }
        Emitter {
            anchors.fill: parent
            emitRate: 4
            lifeSpan: 16000
            lifeSpanVariation: 6000
            size: 6
            sizeVariation: 3
            velocity: PointDirection { y: -30; yVariation: 10; xVariation: 10 }
            y: parent ? parent.height : 0
        }
    }

    // ---- grid ----
    Canvas {
        id: gridCanvas
        anchors.fill: parent
        visible: SettingsManager.backgroundTheme === "grid"
        property real offset: 0

        NumberAnimation on offset {
            running: gridCanvas.visible && !WindowDragState.active
            loops: Animation.Infinite
            from: 0; to: 42
            duration: 8000
        }
        onOffsetChanged: requestPaint()

        onPaint: {
            var ctx = getContext("2d")
            ctx.reset()
            ctx.strokeStyle = Theme.accent
            ctx.globalAlpha = 0.16
            ctx.lineWidth = 1
            var step = 42
            for (var x = -step + (offset % step); x < width; x += step) {
                ctx.beginPath(); ctx.moveTo(x, 0); ctx.lineTo(x, height); ctx.stroke()
            }
            for (var y = -step + (offset % step); y < height; y += step) {
                ctx.beginPath(); ctx.moveTo(0, y); ctx.lineTo(width, y); ctx.stroke()
            }
        }
    }
}
