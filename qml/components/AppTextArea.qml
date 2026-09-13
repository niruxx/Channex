import QtQuick
import QtQuick.Controls.Basic
import Channex

// Theme-consistent drop-in replacement for the Basic style's TextArea
// (which otherwise renders with a white background and black text).
TextArea {
    id: control

    color: Theme.ink
    placeholderTextColor: Theme.inkFaint
    selectionColor: Theme.accent
    selectedTextColor: "#0c0e11"
    font.pixelSize: 13
    selectByMouse: true
    wrapMode: TextArea.Wrap

    leftPadding: 10
    rightPadding: 10
    topPadding: 8
    bottomPadding: 8

    background: Rectangle {
        radius: Theme.radiusSm
        color: Theme.surface2
        border.width: 1
        border.color: control.activeFocus ? Theme.accent : Theme.border

        Behavior on border.color { ColorAnimation { duration: 100 } }
    }
}
