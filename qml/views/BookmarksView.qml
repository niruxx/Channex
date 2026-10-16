import QtQuick
import QtQuick.Controls.Basic
import QtQuick.Layouts
import Channex
import "../components"

// Bookmarked threads as M3 list items on filled cards: thumbnail (or
// a tonal placeholder), subject + site/board, a "new replies" badge,
// and open / remove actions.
Item {
    ColumnLayout {
        anchors.fill: parent
        spacing: 0

        ColumnLayout {
            Layout.fillWidth: true
            Layout.leftMargin: 32
            Layout.topMargin: 28
            Layout.bottomMargin: 16
            spacing: 4
            Text { text: "Bookmarks"; color: Theme.onSurfaceColor; font.pixelSize: Theme.headlineMedium }
            Text {
                text: BookmarksController.bookmarks.length + (BookmarksController.bookmarks.length === 1 ? " saved thread" : " saved threads")
                color: Theme.onSurfaceVariantColor
                font.pixelSize: Theme.bodyLarge
            }
        }

        ListView {
            id: list
            Layout.fillWidth: true
            Layout.fillHeight: true
            leftMargin: 24
            rightMargin: 24
            bottomMargin: 24
            spacing: 8
            clip: true
            model: BookmarksController.bookmarks
            ScrollBar.vertical: AppScrollBar {}

            populate: Transition {
                NumberAnimation { properties: "opacity"; from: 0; to: 1; duration: Theme.durMedium; easing.type: Easing.BezierSpline; easing.bezierCurve: Theme.easeStandard }
            }
            add: Transition {
                NumberAnimation { properties: "opacity"; from: 0; to: 1; duration: Theme.durMedium }
            }
            remove: Transition {
                NumberAnimation { properties: "opacity"; to: 0; duration: Theme.durShort; easing.type: Easing.BezierSpline; easing.bezierCurve: Theme.easeEmphasizedAccel }
                NumberAnimation { properties: "scale"; to: 0.94; duration: Theme.durShort; easing.type: Easing.BezierSpline; easing.bezierCurve: Theme.easeEmphasizedAccel }
            }
            displaced: Transition {
                NumberAnimation { properties: "y"; duration: Theme.durMedium; easing.type: Easing.BezierSpline; easing.bezierCurve: Theme.easeStandard }
            }

            EmptyState {
                anchors.centerIn: parent
                visible: BookmarksController.bookmarks.length === 0
                iconName: "bookmark"
                text: "No bookmarks yet"
                supportingText: "Bookmark a thread from the catalog or thread view to keep track of it here."
            }

            delegate: Rectangle {
                id: bm
                width: list.width - 48
                height: 80
                radius: Theme.shapeMedium
                color: Theme.surfaceContainerLow

                readonly property int newReplies: (modelData.replyCount !== undefined && modelData.lastSeenReplyCount !== undefined)
                    ? Math.max(0, modelData.replyCount - modelData.lastSeenReplyCount) : 0

                HoverHandler { id: bmHover }
                StateLayer { hovered: bmHover.hovered; pressed: bmMouse.pressed }
                MouseArea {
                    id: bmMouse
                    anchors.fill: parent
                    cursorShape: Qt.PointingHandCursor
                    onClicked: NavigationController.goThread(modelData.siteId, modelData.boardCode, modelData.threadId)
                }

                RowLayout {
                    anchors.fill: parent
                    anchors.leftMargin: 12
                    anchors.rightMargin: 12
                    spacing: 16

                    RoundedImage {
                        Layout.preferredWidth: 56
                        Layout.preferredHeight: 56
                        radius: Theme.shapeSmall
                        maskColor: bm.color
                        placeholderColor: Theme.secondaryContainer
                        source: modelData.thumbUrl || ""
                        AppIcon {
                            anchors.centerIn: parent
                            visible: !modelData.thumbUrl
                            name: "bookmark-fill"
                            iconSize: 22
                            color: Theme.onSecondaryContainerColor
                        }
                    }

                    ColumnLayout {
                        Layout.fillWidth: true
                        spacing: 2
                        Text {
                            Layout.fillWidth: true
                            text: modelData.subject && modelData.subject.length ? modelData.subject : ("Thread " + modelData.threadId)
                            color: Theme.onSurfaceColor
                            font.pixelSize: Theme.bodyLarge
                            font.weight: Font.Medium
                            elide: Text.ElideRight
                        }
                        Text {
                            text: modelData.siteId + " · /" + modelData.boardCode + "/ · " + modelData.threadId
                            color: Theme.onSurfaceVariantColor
                            font.pixelSize: Theme.bodyMedium
                        }
                    }

                    Rectangle {
                        visible: bm.newReplies > 0
                        implicitHeight: 24
                        implicitWidth: newLabel.implicitWidth + 16
                        radius: 12
                        color: Theme.primary
                        Text { id: newLabel; anchors.centerIn: parent; text: bm.newReplies + " new"; color: Theme.onPrimaryColor; font.pixelSize: Theme.labelMedium; font.weight: Font.Bold }
                    }

                    AppButton {
                        variant: "icon"
                        iconName: "trash"
                        tooltip: "Remove bookmark"
                        onClicked: BookmarksController.remove(modelData.siteId, modelData.boardCode, modelData.threadId)
                    }
                    AppIcon { name: "chevron-right"; iconSize: 20; color: Theme.onSurfaceVariantColor }
                }
            }
        }
    }
}
