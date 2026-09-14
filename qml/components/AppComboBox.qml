import QtQuick
import QtQuick.Controls.Basic
import Channex

// Theme-consistent drop-in replacement for the Basic style's ComboBox
// (which otherwise renders with a white background/popup and black text).
ComboBox {
    id: control

    implicitHeight: 36
    font.pixelSize: 13

    background: Rectangle {
        radius: Theme.radiusSm
        color: Theme.surface2
        border.width: 1
        border.color: control.popup.visible ? Theme.accent : Theme.border
    }

    contentItem: Text {
        text: control.displayText
        color: Theme.ink
        font: control.font
        leftPadding: 12
        rightPadding: 8
        verticalAlignment: Text.AlignVCenter
        elide: Text.ElideRight
    }

    indicator: Text {
        x: control.width - width - 10
        y: (control.height - height) / 2
        text: "▾"
        color: Theme.inkDim
        font.pixelSize: 11
    }

    delegate: ItemDelegate {
        id: entry
        width: control.width
        highlighted: control.highlightedIndex === index

        contentItem: Text {
            text: modelData
            color: Theme.ink
            font: control.font
            verticalAlignment: Text.AlignVCenter
            elide: Text.ElideRight
        }
        background: Rectangle {
            color: entry.highlighted ? Theme.surface3 : "transparent"
        }
    }

    popup: Popup {
        y: control.height + 4
        width: control.width
        implicitHeight: Math.min(contentItem.implicitHeight + 8, 280)
        padding: 4

        enter: Transition {
            NumberAnimation { properties: "opacity"; from: 0; to: 1; duration: 140; easing.type: Easing.OutCubic }
            NumberAnimation { properties: "scale"; from: 0.94; to: 1; duration: 140; easing.type: Easing.OutCubic }
        }
        exit: Transition {
            NumberAnimation { properties: "opacity"; from: 1; to: 0; duration: 100; easing.type: Easing.InCubic }
        }

        background: Rectangle {
            color: Theme.surface2
            border.width: 1
            border.color: Theme.border
            radius: Theme.radiusSm
            CardShadow { anchors.fill: parent; radius: Theme.radiusSm }
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
