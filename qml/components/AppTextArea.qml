import QtQuick
import QtQuick.Controls.Basic
import Channex

// M3 filled text field, multi-line: same container, active indicator,
// and floating `label` behavior as AppTextField's filled variant.
TextArea {
    id: control

    property string label: ""
    readonly property bool hasLabel: label.length > 0
    readonly property bool floated: activeFocus || text.length > 0 || preeditText.length > 0

    color: Theme.onSurfaceColor
    placeholderTextColor: hasLabel && !floated ? "transparent" : Theme.onSurfaceVariantColor
    selectionColor: Theme.withAlpha(Theme.primary, 0.4)
    selectedTextColor: Theme.onSurfaceColor
    font.pixelSize: Theme.bodyLarge
    selectByMouse: true
    hoverEnabled: true
    wrapMode: TextArea.Wrap

    leftPadding: 16
    rightPadding: 16
    topPadding: hasLabel ? 26 : 14
    bottomPadding: 12

    background: Rectangle {
        color: Theme.surfaceContainerHighest
        topLeftRadius: Theme.shapeExtraSmall
        topRightRadius: Theme.shapeExtraSmall

        StateLayer {
            radius: 0
            layerColor: Theme.onSurfaceColor
            hovered: control.hovered && !control.activeFocus
        }

        Rectangle {
            anchors.left: parent.left
            anchors.right: parent.right
            anchors.bottom: parent.bottom
            height: control.activeFocus ? 2 : 1
            color: control.activeFocus ? Theme.primary : Theme.onSurfaceVariantColor
            Behavior on color { ColorAnimation { duration: Theme.durShort } }
        }

        Text {
            visible: control.hasLabel
            x: control.leftPadding
            y: control.floated ? 8 : 16
            text: control.label
            color: control.activeFocus ? Theme.primary : Theme.onSurfaceVariantColor
            font.pixelSize: control.floated ? Theme.bodySmall : Theme.bodyLarge

            Behavior on y { NumberAnimation { duration: Theme.durShort; easing.type: Easing.BezierSpline; easing.bezierCurve: Theme.easeStandard } }
            Behavior on font.pixelSize { NumberAnimation { duration: Theme.durShort; easing.type: Easing.BezierSpline; easing.bezierCurve: Theme.easeStandard } }
            Behavior on color { ColorAnimation { duration: Theme.durShort } }
        }
    }
}
