import QtQuick
import QtQuick.Layouts
import Channex

// M3 navigation drawer item: 48px row with a full-pill active
// indicator (secondaryContainer) behind the selected destination, an
// optional leading icon, label, and trailing badge text.
Rectangle {
    id: root
    property string text: ""
    property string secondaryText: ""
    property string iconName: ""
    property string badge: ""
    property bool selected: false
    signal clicked()
    signal rightClicked()

    implicitHeight: 44
    radius: height / 2
    color: selected ? Theme.secondaryContainer : "transparent"
    Behavior on color { ColorAnimation { duration: Theme.durShort } }

    StateLayer {
        layerColor: root.selected ? Theme.onSecondaryContainerColor : Theme.onSurfaceColor
        hovered: hover.hovered
        pressed: mouse.pressed
    }

    RowLayout {
        anchors.fill: parent
        anchors.leftMargin: 16
        anchors.rightMargin: 20
        spacing: 12

        AppIcon {
            visible: root.iconName.length > 0
            name: root.iconName
            iconSize: 20
            color: root.selected ? Theme.onSecondaryContainerColor : Theme.onSurfaceVariantColor
        }

        Text {
            text: root.text
            color: root.selected ? Theme.onSecondaryContainerColor : Theme.onSurfaceVariantColor
            font.pixelSize: Theme.labelLarge
            font.weight: root.selected ? Font.Bold : Font.Medium
        }

        Text {
            Layout.fillWidth: true
            visible: root.secondaryText.length > 0
            text: root.secondaryText
            color: root.selected ? Theme.onSecondaryContainerColor : Theme.onSurfaceVariantColor
            opacity: 0.8
            font.pixelSize: Theme.labelLarge
            elide: Text.ElideRight
        }

        Item { Layout.fillWidth: true; visible: root.secondaryText.length === 0 }

        Text {
            visible: root.badge.length > 0
            text: root.badge
            color: root.selected ? Theme.onSecondaryContainerColor : Theme.onSurfaceVariantColor
            font.pixelSize: Theme.labelLarge
            font.weight: Font.Medium
        }
    }

    HoverHandler { id: hover }
    MouseArea {
        id: mouse
        anchors.fill: parent
        cursorShape: Qt.PointingHandCursor
        acceptedButtons: Qt.LeftButton | Qt.RightButton
        onClicked: (m) => m.button === Qt.RightButton ? root.rightClicked() : root.clicked()
    }
}
