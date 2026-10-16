import QtQuick
import QtQuick.Controls.Basic
import Channex

// M3 exposed dropdown menu, filled style: looks like AppTextField's
// filled variant (optional floating `label` that sits permanently
// floated, since a combo box always shows a value) with a trailing
// arrow that flips while the menu is open. The popup is an M3 menu.
ComboBox {
    id: control

    property string label: ""
    readonly property bool hasLabel: label.length > 0

    implicitHeight: hasLabel ? 56 : 48
    font.pixelSize: Theme.bodyLarge
    hoverEnabled: true

    background: Rectangle {
        color: Theme.surfaceContainerHighest
        topLeftRadius: Theme.shapeExtraSmall
        topRightRadius: Theme.shapeExtraSmall

        StateLayer {
            radius: 0
            layerColor: Theme.onSurfaceColor
            hovered: control.hovered && !control.popup.visible
        }

        Rectangle {
            anchors.left: parent.left
            anchors.right: parent.right
            anchors.bottom: parent.bottom
            height: control.popup.visible || control.visualFocus ? 2 : 1
            color: control.popup.visible || control.visualFocus ? Theme.primary : Theme.onSurfaceVariantColor
        }

        Text {
            visible: control.hasLabel
            x: 16
            y: 8
            text: control.label
            color: control.popup.visible ? Theme.primary : Theme.onSurfaceVariantColor
            font.pixelSize: Theme.bodySmall
        }
    }

    contentItem: Text {
        text: control.displayText
        color: Theme.onSurfaceColor
        font: control.font
        leftPadding: 16
        rightPadding: 8
        topPadding: control.hasLabel ? 18 : 0
        verticalAlignment: Text.AlignVCenter
        elide: Text.ElideRight
    }

    indicator: AppIcon {
        x: control.width - width - 14
        y: (control.height - height) / 2 + (control.hasLabel ? 0 : 0)
        name: "chevron-down"
        iconSize: 20
        color: control.popup.visible ? Theme.primary : Theme.onSurfaceVariantColor
        rotation: control.popup.visible ? 180 : 0
        Behavior on rotation { NumberAnimation { duration: Theme.durShort; easing.type: Easing.BezierSpline; easing.bezierCurve: Theme.easeStandard } }
    }

    delegate: ItemDelegate {
        id: entry
        width: control.width
        implicitHeight: 44
        highlighted: control.highlightedIndex === index
        hoverEnabled: true

        contentItem: Text {
            text: modelData
            color: Theme.onSurfaceColor
            font.pixelSize: Theme.bodyLarge
            leftPadding: 4
            verticalAlignment: Text.AlignVCenter
            elide: Text.ElideRight
        }
        background: Rectangle {
            color: control.currentIndex === index ? Theme.secondaryContainer : "transparent"
            StateLayer {
                radius: 0
                layerColor: Theme.onSurfaceColor
                hovered: entry.hovered || entry.highlighted
                pressed: entry.down
            }
        }
    }

    popup: Popup {
        y: control.height + 2
        width: control.width
        implicitHeight: Math.min(contentItem.implicitHeight + 16, 320)
        topPadding: 8
        bottomPadding: 8
        leftPadding: 0
        rightPadding: 0

        enter: Transition {
            NumberAnimation { property: "opacity"; from: 0; to: 1; duration: Theme.durShort; easing.type: Easing.BezierSpline; easing.bezierCurve: Theme.easeEmphasizedDecel }
            NumberAnimation { property: "scale"; from: 0.95; to: 1; duration: Theme.durShort; easing.type: Easing.BezierSpline; easing.bezierCurve: Theme.easeEmphasizedDecel }
        }
        exit: Transition {
            NumberAnimation { property: "opacity"; from: 1; to: 0; duration: 100 }
        }

        background: Rectangle {
            color: Theme.surfaceContainer
            radius: Theme.shapeExtraSmall
            CardShadow { radius: parent.radius; level: 2 }
        }

        contentItem: ListView {
            clip: true
            implicitHeight: contentHeight
            model: control.popup.visible ? control.delegateModel : null
            currentIndex: control.highlightedIndex
            ScrollIndicator.vertical: ScrollIndicator {}
        }
    }
}
