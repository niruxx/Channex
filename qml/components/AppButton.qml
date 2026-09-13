import QtQuick
import QtQuick.Controls.Basic
import Channex

// Theme-consistent drop-in replacement for the Basic style's Button,
// which otherwise renders with its light default palette (white/grey)
// regardless of Theme.dark - every raw `Button {}` in a dark-themed view
// should use this instead. Set `iconName` (see resources/icons/ui/) for
// a leading icon; leave `text` empty alongside it for an icon-only
// square button, or set both for an icon+label button.
Button {
    id: control

    property string iconName: ""
    readonly property bool iconOnly: iconName.length > 0 && text.length === 0
    readonly property bool hasIcon: iconName.length > 0

    readonly property color bgColor: control.flat
        ? "transparent"
        : (control.checked ? Theme.accent : Theme.surface3)
    readonly property color bgHoverColor: control.flat
        ? Theme.surface2
        : (control.checked ? Theme.accent : Theme.surface4)
    readonly property color fgColor: control.checked ? (control.flat ? Theme.accent : "#0c0e11") : Theme.ink

    implicitHeight: 36
    implicitWidth: iconOnly ? 36 : implicitContentWidth + leftPadding + rightPadding
    leftPadding: iconOnly ? 0 : (hasIcon ? 12 : 14)
    rightPadding: iconOnly ? 0 : 14
    font.pixelSize: 13

    scale: control.down ? 0.96 : 1.0
    Behavior on scale { NumberAnimation { duration: 90; easing.type: Easing.OutCubic } }

    background: Rectangle {
        radius: Theme.radiusSm
        color: !control.enabled ? Theme.surface2
             : control.down ? Qt.darker(control.hovered ? control.bgHoverColor : control.bgColor, 1.15)
             : (control.hovered ? control.bgHoverColor : control.bgColor)
        border.width: control.flat ? 0 : 1
        border.color: Theme.border

        Behavior on color { ColorAnimation { duration: 100 } }
    }

    contentItem: Row {
        spacing: 8

        AppIcon {
            visible: control.hasIcon
            anchors.verticalCenter: parent.verticalCenter
            name: control.iconName
            iconSize: 15
            color: control.enabled ? control.fgColor : Theme.inkFaint
        }

        Text {
            visible: control.text.length > 0
            anchors.verticalCenter: parent.verticalCenter
            text: control.text
            font: control.font
            color: control.enabled ? control.fgColor : Theme.inkFaint
            horizontalAlignment: Text.AlignHCenter
            verticalAlignment: Text.AlignVCenter
            elide: Text.ElideRight
        }
    }
}
