import QtQuick
import QtQuick.Layouts
import Channex

// A titled settings group: tonal icon disc + titleMedium heading, then
// a large-radius surfaceContainerLow container holding the rows
// (children go straight into its column).
ColumnLayout {
    id: root
    property string title: ""
    property string iconName: ""
    default property alias rows: body.data
    spacing: 12

    RowLayout {
        Layout.leftMargin: 4
        spacing: 12
        Rectangle {
            implicitWidth: 36; implicitHeight: 36; radius: 18
            color: Theme.secondaryContainer
            AppIcon { anchors.centerIn: parent; name: root.iconName; iconSize: 18; color: Theme.onSecondaryContainerColor }
        }
        Text { text: root.title; color: Theme.onSurfaceColor; font.pixelSize: Theme.titleMedium; font.weight: Font.Medium }
    }

    Rectangle {
        Layout.fillWidth: true
        implicitHeight: body.implicitHeight + 16
        radius: Theme.shapeLargeIncreased
        color: Theme.surfaceContainerLow

        ColumnLayout {
            id: body
            x: 20
            y: 8
            width: parent.width - 40
            spacing: 0
        }
    }
}
