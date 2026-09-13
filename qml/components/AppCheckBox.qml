import QtQuick
import QtQuick.Controls.Basic
import Channex

// Theme-consistent drop-in replacement for the Basic style's CheckBox
// (which otherwise renders with a white/grey box that barely reads on
// a dark background).
CheckBox {
    id: control

    font.pixelSize: 13

    indicator: Rectangle {
        x: control.leftPadding
        y: (control.height - height) / 2
        width: 18; height: 18
        radius: 4
        color: control.checked ? Theme.accent : Theme.surface2
        border.width: 1
        border.color: control.checked ? Theme.accent : Theme.border

        Behavior on color { ColorAnimation { duration: 100 } }

        Text {
            anchors.centerIn: parent
            visible: control.checked
            text: "✓"
            color: "#0c0e11"
            font.pixelSize: 12
            font.bold: true
        }
    }

    contentItem: Text {
        text: control.text
        color: Theme.ink
        font: control.font
        leftPadding: control.indicator.width + 8
        verticalAlignment: Text.AlignVCenter
    }
}
