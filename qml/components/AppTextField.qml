import QtQuick
import QtQuick.Controls.Basic
import Channex

// Theme-consistent drop-in replacement for the Basic style's TextField
// (which otherwise renders with a white background and black text).
// Set `leftIcon` (see resources/icons/ui/) to show a leading glyph.
TextField {
    id: control

    property string leftIcon: ""

    color: Theme.ink
    placeholderTextColor: Theme.inkFaint
    selectionColor: Theme.accent
    selectedTextColor: "#0c0e11"
    font.pixelSize: 13
    selectByMouse: true

    leftPadding: control.leftIcon.length > 0 ? 34 : 12
    rightPadding: 12
    topPadding: 8
    bottomPadding: 8
    implicitHeight: 36

    background: Rectangle {
        radius: Theme.radiusSm
        color: Theme.surface2
        border.width: 1
        border.color: control.activeFocus ? Theme.accent : Theme.border

        Behavior on border.color { ColorAnimation { duration: 100 } }

        AppIcon {
            visible: control.leftIcon.length > 0
            anchors.left: parent.left
            anchors.leftMargin: 10
            anchors.verticalCenter: parent.verticalCenter
            name: control.leftIcon
            iconSize: 14
            color: Theme.inkFaint
        }
    }
}
