import QtQuick
import QtQuick.Controls.Basic
import Channex

// Theme-consistent drop-in replacement for the Basic style's Button,
// which otherwise renders with its light default palette (white/grey)
// regardless of Theme.dark - every raw `Button {}` in a dark-themed view
// should use this instead.
Button {
    id: control

    readonly property color bgColor: control.flat
        ? "transparent"
        : (control.checked ? Theme.accent : Theme.surface3)
    readonly property color bgHoverColor: control.flat
        ? Theme.surface2
        : (control.checked ? Theme.accent : Theme.surface4)
    readonly property color fgColor: control.checked ? (control.flat ? Theme.accent : "#0c0e11") : Theme.ink

    implicitHeight: 36
    leftPadding: 14
    rightPadding: 14
    font.pixelSize: 13

    background: Rectangle {
        radius: Theme.radiusSm
        color: !control.enabled ? Theme.surface2
             : control.down ? Qt.darker(control.hovered ? control.bgHoverColor : control.bgColor, 1.15)
             : (control.hovered ? control.bgHoverColor : control.bgColor)
        border.width: control.flat ? 0 : 1
        border.color: Theme.border

        Behavior on color { ColorAnimation { duration: 100 } }
    }

    contentItem: Text {
        text: control.text
        font: control.font
        color: control.enabled ? control.fgColor : Theme.inkFaint
        horizontalAlignment: Text.AlignHCenter
        verticalAlignment: Text.AlignVCenter
        elide: Text.ElideRight
    }
}
