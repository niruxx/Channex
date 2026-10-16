import QtQuick
import QtQuick.Controls.Basic
import Channex

// M3 text field. Two looks:
//
//   variant "filled" (default) - surfaceContainerHighest fill, square
//     bottom with an active indicator line (1px, 2px primary on focus),
//     and an optional floating `label` that rests inside the field and
//     floats up to the top on focus / once there's text.
//   variant "search" - the M3 search bar: a full-pill
//     surfaceContainerHigh field with a leading icon.
//
// `leftIcon` (resources/icons/ui/) adds a leading glyph to either.
TextField {
    id: control

    property string label: ""
    property string leftIcon: variant === "search" ? "search" : ""
    property string variant: "filled"

    readonly property bool isSearch: variant === "search"
    readonly property bool hasLabel: label.length > 0 && !isSearch
    readonly property bool floated: activeFocus || text.length > 0 || preeditText.length > 0

    color: Theme.onSurfaceColor
    placeholderTextColor: hasLabel && !floated ? "transparent" : Theme.onSurfaceVariantColor
    selectionColor: Theme.withAlpha(Theme.primary, 0.4)
    selectedTextColor: Theme.onSurfaceColor
    font.pixelSize: Theme.bodyLarge
    selectByMouse: true
    hoverEnabled: true
    verticalAlignment: TextInput.AlignVCenter

    leftPadding: leftIcon.length > 0 ? (isSearch ? 52 : 48) : 16
    rightPadding: 16
    topPadding: hasLabel ? 22 : 0
    bottomPadding: hasLabel ? 6 : 0
    implicitHeight: isSearch ? 48 : (hasLabel ? 56 : 48)

    background: Rectangle {
        color: control.isSearch ? Theme.surfaceContainerHigh : Theme.surfaceContainerHighest
        radius: control.isSearch ? height / 2 : 0
        topLeftRadius: control.isSearch ? height / 2 : Theme.shapeExtraSmall
        topRightRadius: control.isSearch ? height / 2 : Theme.shapeExtraSmall
        opacity: control.enabled ? 1 : 0.5

        StateLayer {
            radius: parent.radius
            layerColor: Theme.onSurfaceColor
            hovered: control.hovered && !control.activeFocus
        }

        // Active indicator (filled variant only).
        Rectangle {
            visible: !control.isSearch
            anchors.left: parent.left
            anchors.right: parent.right
            anchors.bottom: parent.bottom
            height: control.activeFocus ? 2 : 1
            color: control.activeFocus ? Theme.primary : Theme.onSurfaceVariantColor
            Behavior on color { ColorAnimation { duration: Theme.durShort } }
        }

        // Search bar focus ring.
        Rectangle {
            visible: control.isSearch
            anchors.fill: parent
            radius: parent.radius
            color: "transparent"
            border.width: 2
            border.color: Theme.primary
            opacity: control.activeFocus ? 1 : 0
            Behavior on opacity { NumberAnimation { duration: Theme.durShort } }
        }

        AppIcon {
            visible: control.leftIcon.length > 0
            anchors.left: parent.left
            anchors.leftMargin: control.isSearch ? 18 : 14
            anchors.verticalCenter: parent.verticalCenter
            name: control.leftIcon
            iconSize: 20
            color: Theme.onSurfaceVariantColor
        }

        Text {
            id: floatingLabel
            visible: control.hasLabel
            x: control.leftPadding
            text: control.label
            color: control.activeFocus ? Theme.primary : Theme.onSurfaceVariantColor
            font.pixelSize: control.floated ? Theme.bodySmall : Theme.bodyLarge
            y: control.floated ? 8 : (parent.height - height) / 2

            Behavior on y { NumberAnimation { duration: Theme.durShort; easing.type: Easing.BezierSpline; easing.bezierCurve: Theme.easeStandard } }
            Behavior on font.pixelSize { NumberAnimation { duration: Theme.durShort; easing.type: Easing.BezierSpline; easing.bezierCurve: Theme.easeStandard } }
            Behavior on color { ColorAnimation { duration: Theme.durShort } }
        }
    }
}
