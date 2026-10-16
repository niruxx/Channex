import QtQuick
import Channex

// M3 circular progress indicator (indeterminate): a primary-colored arc
// that grows and shrinks while the whole thing rotates.
Item {
    id: root
    property bool running: true
    property color color: Theme.primary
    property real lineWidth: 4
    implicitWidth: 44
    implicitHeight: 44
    visible: running
    opacity: running ? 1 : 0

    property real sweep: 0.1

    SequentialAnimation on sweep {
        running: root.running && root.visible
        loops: Animation.Infinite
        NumberAnimation { from: 0.08; to: 0.75; duration: 700; easing.type: Easing.InOutCubic }
        NumberAnimation { from: 0.75; to: 0.08; duration: 700; easing.type: Easing.InOutCubic }
    }

    Canvas {
        id: canvas
        anchors.fill: parent
        RotationAnimator on rotation {
            running: root.running && root.visible
            loops: Animation.Infinite
            from: 0; to: 360
            duration: 1100
        }
        onPaint: {
            var ctx = getContext("2d")
            ctx.reset()
            var r = (Math.min(width, height) - root.lineWidth) / 2
            ctx.lineWidth = root.lineWidth
            ctx.lineCap = "round"
            ctx.strokeStyle = root.color
            ctx.beginPath()
            ctx.arc(width / 2, height / 2, r, 0, Math.PI * 2 * root.sweep)
            ctx.stroke()
        }
    }

    onSweepChanged: canvas.requestPaint()
    onColorChanged: canvas.requestPaint()
}
