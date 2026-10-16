import QtQuick
import QtQuick.Controls.Basic
import Channex

// M3 switch: 52x32 track, a handle that grows from 16 to 24 when on
// (28 while pressed), with a check glyph inside the "on" handle.
Switch {
    id: control
    hoverEnabled: true
    padding: 0
    spacing: 12
    font.pixelSize: Theme.bodyLarge

    indicator: Rectangle {
        implicitWidth: 52
        implicitHeight: 32
        x: control.text.length > 0 ? control.width - width : 0
        y: (control.height - height) / 2
        radius: 16
        color: control.checked ? Theme.primary : Theme.surfaceContainerHighest
        border.width: control.checked ? 0 : 2
        border.color: Theme.outline
        opacity: control.enabled ? 1 : 0.38
        Behavior on color { ColorAnimation { duration: Theme.durShort } }

        Rectangle {
            id: handleHalo
            width: 40; height: 40; radius: 20
            anchors.centerIn: handle
            color: control.checked ? Theme.primary : Theme.onSurfaceColor
            opacity: control.pressed ? Theme.pressedOpacity : (control.hovered ? Theme.hoverOpacity : 0)
            Behavior on opacity { NumberAnimation { duration: Theme.durShort } }
        }

        Rectangle {
            id: handle
            property real d: control.pressed ? 28 : (control.checked ? 24 : 16)
            width: d; height: d; radius: d / 2
            anchors.verticalCenter: parent.verticalCenter
            x: control.checked ? parent.width - 16 - d / 2 : 16 - d / 2
            color: control.checked ? Theme.onPrimaryColor : Theme.outline

            Behavior on x { NumberAnimation { duration: Theme.durMedium; easing.type: Easing.BezierSpline; easing.bezierCurve: Theme.easeEmphasizedDecel } }
            Behavior on d { NumberAnimation { duration: Theme.durShort; easing.type: Easing.BezierSpline; easing.bezierCurve: Theme.easeStandard } }
            Behavior on color { ColorAnimation { duration: Theme.durShort } }

            AppIcon {
                anchors.centerIn: parent
                visible: control.checked
                name: "check"
                iconSize: 14
                color: Theme.onPrimaryContainerColor
            }
        }
    }

    contentItem: Text {
        text: control.text
        visible: text.length > 0
        color: Theme.onSurfaceColor
        font: control.font
        rightPadding: control.indicator.width + control.spacing
        verticalAlignment: Text.AlignVCenter
        wrapMode: Text.WordWrap
    }
}
