import QtQuick
import Channex

// M3 chip: 32px, 8px corner radius. As a filter chip (`checkable`),
// the selected state fills with secondaryContainer and shows a leading
// check; otherwise an optional leading icon. `swatch` draws a small
// color dot in place of the icon (used for theme previews).
Rectangle {
    id: root
    property string text: ""
    property string iconName: ""
    property color swatch: "transparent"
    property bool checkable: false
    property bool checked: false
    signal clicked()

    implicitHeight: 32
    implicitWidth: chipRow.implicitWidth + (leading ? 24 : 32)
    radius: Theme.shapeSmall
    color: checked ? Theme.secondaryContainer : "transparent"
    border.width: checked ? 0 : 1
    border.color: Theme.outlineVariant

    readonly property bool leading: (checkable && checked) || iconName.length > 0 || swatch.a > 0

    Behavior on color { ColorAnimation { duration: Theme.durShort } }

    StateLayer {
        layerColor: root.checked ? Theme.onSecondaryContainerColor : Theme.onSurfaceVariantColor
        hovered: chipHover.hovered
        pressed: chipMouse.pressed
    }

    Row {
        id: chipRow
        anchors.centerIn: parent
        spacing: 8

        Rectangle {
            visible: root.swatch.a > 0 && !(root.checkable && root.checked)
            anchors.verticalCenter: parent.verticalCenter
            width: 14; height: 14; radius: 7
            color: root.swatch
            border.width: 1
            border.color: Theme.withAlpha(Theme.onSurfaceColor, 0.2)
        }
        AppIcon {
            visible: (root.checkable && root.checked) || (root.iconName.length > 0 && root.swatch.a === 0)
            anchors.verticalCenter: parent.verticalCenter
            name: root.checkable && root.checked ? "check" : root.iconName
            iconSize: 18
            color: root.checked ? Theme.onSecondaryContainerColor : Theme.primary
        }
        Text {
            anchors.verticalCenter: parent.verticalCenter
            text: root.text
            color: root.checked ? Theme.onSecondaryContainerColor : Theme.onSurfaceVariantColor
            font.pixelSize: Theme.labelLarge
            font.weight: Font.Medium
        }
    }

    HoverHandler { id: chipHover }
    MouseArea {
        id: chipMouse
        anchors.fill: parent
        cursorShape: Qt.PointingHandCursor
        onClicked: root.clicked()
    }
}
