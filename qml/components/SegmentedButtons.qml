import QtQuick
import QtQuick.Layouts
import Channex

// M3 segmented button (single-select). `model` is a list of
// { value, label, icon } (label and icon each optional); the segment
// whose value equals `current` is selected, and gets a leading check
// mark when it has a text label. Emits `activated(value)`.
Rectangle {
    id: root
    property var model: []
    property var current
    property bool compact: false
    signal activated(var value)

    implicitHeight: compact ? 32 : 40
    implicitWidth: segments.implicitWidth
    radius: height / 2
    color: "transparent"
    border.width: 1
    border.color: Theme.outline
    clip: true

    RowLayout {
        id: segments
        anchors.fill: parent
        spacing: 0

        Repeater {
            model: root.model
            delegate: Item {
                id: seg
                readonly property bool selected: root.current === modelData.value
                readonly property bool hasLabel: modelData.label !== undefined && modelData.label.length > 0
                Layout.fillWidth: true
                Layout.fillHeight: true
                implicitWidth: segRow.implicitWidth + (hasLabel ? 32 : 28)

                // Selected fill + hover layer. Each segment is a plain
                // rect; the outer pill's clip rounds the two end ones.
                Rectangle {
                    anchors.fill: parent
                    color: seg.selected ? Theme.secondaryContainer : "transparent"
                    Behavior on color { ColorAnimation { duration: Theme.durShort } }
                    StateLayer {
                        radius: 0
                        layerColor: seg.selected ? Theme.onSecondaryContainerColor : Theme.onSurfaceColor
                        hovered: segHover.hovered
                        pressed: segMouse.pressed
                    }
                }

                // Divider to the left of every segment but the first.
                Rectangle {
                    visible: index > 0
                    width: 1
                    height: parent.height
                    color: Theme.outline
                }

                Row {
                    id: segRow
                    anchors.centerIn: parent
                    spacing: 8

                    AppIcon {
                        anchors.verticalCenter: parent.verticalCenter
                        visible: (seg.selected && seg.hasLabel) || (modelData.icon !== undefined && modelData.icon.length > 0)
                        name: seg.selected && seg.hasLabel ? "check" : (modelData.icon || "")
                        iconSize: root.compact ? 16 : 18
                        color: seg.selected ? Theme.onSecondaryContainerColor : Theme.onSurfaceColor
                    }
                    Text {
                        anchors.verticalCenter: parent.verticalCenter
                        visible: seg.hasLabel
                        text: modelData.label || ""
                        color: seg.selected ? Theme.onSecondaryContainerColor : Theme.onSurfaceColor
                        font.pixelSize: root.compact ? Theme.labelMedium : Theme.labelLarge
                        font.weight: Font.Medium
                    }
                }

                HoverHandler { id: segHover }
                MouseArea {
                    id: segMouse
                    anchors.fill: parent
                    cursorShape: Qt.PointingHandCursor
                    onClicked: root.activated(modelData.value)
                }
            }
        }
    }
}
