import QtQuick
import QtQuick.Layouts
import Channex

// One row of a SettingsSection: headline + optional supporting text on
// the left, and either a trailing control (`trailing`, e.g. a switch)
// or a full-width control underneath (children). Rows after the first
// get a hairline divider.
ColumnLayout {
    id: root
    property string title: ""
    property string supportingText: ""
    property bool divider: true
    property alias trailing: trailingSlot.data
    default property alias below: belowSlot.data
    Layout.fillWidth: true
    spacing: 0

    Rectangle {
        visible: root.divider
        Layout.fillWidth: true
        height: 1
        color: Theme.outlineVariant
        opacity: 0.6
    }

    RowLayout {
        Layout.fillWidth: true
        Layout.topMargin: 14
        Layout.bottomMargin: belowSlot.children.length > 0 ? 10 : 14
        spacing: 16

        ColumnLayout {
            Layout.fillWidth: true
            spacing: 2
            Text {
                Layout.fillWidth: true
                text: root.title
                color: Theme.onSurfaceColor
                font.pixelSize: Theme.bodyLarge
                wrapMode: Text.WordWrap
            }
            Text {
                Layout.fillWidth: true
                visible: root.supportingText.length > 0
                text: root.supportingText
                color: Theme.onSurfaceVariantColor
                font.pixelSize: Theme.bodyMedium
                wrapMode: Text.WordWrap
            }
        }

        Item {
            id: trailingSlot
            implicitWidth: childrenRect.width
            implicitHeight: childrenRect.height
        }
    }

    ColumnLayout {
        id: belowSlot
        Layout.fillWidth: true
        Layout.bottomMargin: children.length > 0 ? 16 : 0
        spacing: 8
    }
}
