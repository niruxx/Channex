import QtQuick
import QtQuick.Controls.Basic
import Channex
import "../components"

// Mirrors src/components/bookmarks/BookmarksPanel.tsx.
Item {
    ListView {
        anchors.fill: parent
        anchors.margins: 12
        spacing: 8
        clip: true
        model: BookmarksController.bookmarks

        populate: Transition {
            NumberAnimation { properties: "opacity"; from: 0; to: 1; duration: 200; easing.type: Easing.OutCubic }
        }
        add: Transition {
            NumberAnimation { properties: "opacity"; from: 0; to: 1; duration: 180; easing.type: Easing.OutCubic }
        }
        remove: Transition {
            NumberAnimation { properties: "opacity"; to: 0; duration: 140; easing.type: Easing.InCubic }
            NumberAnimation { properties: "scale"; to: 0.9; duration: 140; easing.type: Easing.InCubic }
        }
        displaced: Transition {
            NumberAnimation { properties: "y"; duration: 180; easing.type: Easing.OutCubic }
        }

        Text {
            anchors.centerIn: parent
            visible: BookmarksController.bookmarks.length === 0
            text: "No bookmarks yet - star a thread from the catalog or thread view."
            color: Theme.inkFaint
        }

        delegate: Rectangle {
            width: ListView.view.width
            height: 64
            radius: Theme.radiusMd
            color: Theme.surface2
            border.color: Theme.border

            Row {
                anchors.fill: parent
                anchors.margins: 10
                spacing: 10

                Column {
                    width: parent.width - 90
                    Text {
                        text: modelData.subject && modelData.subject.length ? modelData.subject : ("Thread #" + modelData.threadId)
                        color: Theme.ink
                        font.bold: true
                        font.pixelSize: 13
                        elide: Text.ElideRight
                        width: parent.width
                    }
                    Text {
                        text: modelData.siteId + " / " + modelData.boardCode
                        color: Theme.inkFaint
                        font.pixelSize: 11
                    }
                }
            }

            Row {
                anchors.right: parent.right
                anchors.verticalCenter: parent.verticalCenter
                anchors.rightMargin: 10
                spacing: 6

                AppButton {
                    text: "Open"
                    flat: true
                    checked: true
                    onClicked: NavigationController.goThread(modelData.siteId, modelData.boardCode, modelData.threadId)
                }
                AppButton {
                    text: "✕"
                    flat: true
                    onClicked: BookmarksController.remove(modelData.siteId, modelData.boardCode, modelData.threadId)
                }
            }
        }
    }
}
