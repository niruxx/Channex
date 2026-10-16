import QtQuick
import QtQuick.Layouts
import Channex
import "components"

// Centered illustration-less empty state: a large tonal icon disc,
// a title, and optional supporting text.
ColumnLayout {
    id: root
    property string text: ""
    property string supportingText: ""
    property string iconName: "grid"
    property bool isError: false
    spacing: 12

    Rectangle {
        Layout.alignment: Qt.AlignHCenter
        implicitWidth: 72; implicitHeight: 72; radius: 36
        color: root.isError ? Theme.errorContainer : Theme.secondaryContainer
        AppIcon {
            anchors.centerIn: parent
            name: root.iconName
            iconSize: 32
            color: root.isError ? Theme.onErrorContainerColor : Theme.onSecondaryContainerColor
        }
    }
    Text {
        Layout.alignment: Qt.AlignHCenter
        Layout.maximumWidth: 420
        text: root.text
        color: Theme.onSurfaceColor
        font.pixelSize: Theme.titleMedium
        font.weight: Font.Medium
        horizontalAlignment: Text.AlignHCenter
        wrapMode: Text.WordWrap
    }
    Text {
        Layout.alignment: Qt.AlignHCenter
        Layout.maximumWidth: 420
        visible: root.supportingText.length > 0
        text: root.supportingText
        color: Theme.onSurfaceVariantColor
        font.pixelSize: Theme.bodyMedium
        horizontalAlignment: Text.AlignHCenter
        wrapMode: Text.WordWrap
    }
}
