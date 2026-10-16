import QtQuick
import QtQuick.Controls.Basic
import Channex

// M3 menu item: 48px row, labelLarge-ish body text, optional leading
// icon, onSurfaceColor state layer on hover/press.
MenuItem {
    id: control
    property string iconName: ""
    implicitHeight: 44
    leftPadding: 12
    rightPadding: 12
    hoverEnabled: true
    font.pixelSize: Theme.bodyMedium

    contentItem: Row {
        spacing: 12
        AppIcon {
            visible: control.iconName.length > 0
            anchors.verticalCenter: parent.verticalCenter
            name: control.iconName
            iconSize: 20
            color: Theme.onSurfaceVariantColor
        }
        Text {
            anchors.verticalCenter: parent.verticalCenter
            text: control.text
            color: control.enabled ? Theme.onSurfaceColor : Theme.withAlpha(Theme.onSurfaceColor, Theme.disabledContentOpacity)
            font: control.font
        }
    }

    background: Rectangle {
        color: "transparent"
        StateLayer {
            layerColor: Theme.onSurfaceColor
            hovered: control.hovered || control.highlighted
            pressed: control.down
        }
    }
}
