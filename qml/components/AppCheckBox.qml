import QtQuick
import QtQuick.Controls.Basic
import Channex

// M3 checkbox: 18px box with a 2px corner radius, outlined when off,
// primary-filled with an onPrimaryColor check when on, inside a 40px round
// state-layer target.
CheckBox {
    id: control
    hoverEnabled: true
    font.pixelSize: Theme.bodyMedium
    leftPadding: 0
    spacing: 4

    indicator: Item {
        implicitWidth: 40
        implicitHeight: 40
        x: control.leftPadding
        y: (control.height - height) / 2

        Rectangle {
            anchors.fill: parent
            radius: 20
            color: control.checked ? Theme.primary : Theme.onSurfaceColor
            opacity: control.pressed ? Theme.pressedOpacity : (control.hovered ? Theme.hoverOpacity : 0)
            Behavior on opacity { NumberAnimation { duration: Theme.durShort } }
        }

        Rectangle {
            anchors.centerIn: parent
            width: 18; height: 18
            radius: 2
            color: control.checked ? Theme.primary : "transparent"
            border.width: control.checked ? 0 : 2
            border.color: Theme.onSurfaceVariantColor
            opacity: control.enabled ? 1 : 0.38
            Behavior on color { ColorAnimation { duration: Theme.durShort } }

            AppIcon {
                anchors.centerIn: parent
                visible: control.checked
                name: "check"
                iconSize: 14
                color: Theme.onPrimaryColor
            }
        }
    }

    contentItem: Text {
        text: control.text
        color: Theme.onSurfaceColor
        font: control.font
        leftPadding: control.indicator.width + control.spacing
        verticalAlignment: Text.AlignVCenter
        elide: Text.ElideRight
    }
}
