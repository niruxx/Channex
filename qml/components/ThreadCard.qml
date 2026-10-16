import QtQuick
import QtQuick.Controls.Basic
import QtQuick.Layouts
import Channex

// Catalog grid item as an M3 elevated card: surfaceContainerLow,
// medium (12px) corners, a rounded thumbnail inset at the top, then
// subject / excerpt / reply-image counts. Hover adds the state layer
// and lifts the elevation; sticky/closed show as small badges on the
// thumbnail, and the bookmark toggle is an icon button over it.
Rectangle {
    id: root
    required property var post
    property bool compact: false
    property var site: ({})

    signal opened()

    radius: Theme.shapeMedium
    color: Theme.surfaceContainerLow

    // post itself has no thumbUrl/spoiler - those live on its first file.
    readonly property var firstFile: (post.files && post.files.length > 0) ? post.files[0] : null
    readonly property bool shouldBlur: (site.nsfw || (firstFile && firstFile.spoiler)) && SettingsManager.blurNsfw && !revealed
    property bool revealed: false

    // Reference bookmarks.length so this re-evaluates on bookmarksChanged
    // (isBookmarked() alone is a plain invokable call QML won't re-bind on).
    readonly property bool bookmarked: BookmarksController.bookmarks.length >= 0
        && BookmarksController.isBookmarked(site.id, NavigationController.boardCode, String(post.threadId))

    CardShadow { radius: root.radius; level: cardHover.hovered ? 2 : 1 }

    HoverHandler { id: cardHover }

    MouseArea {
        anchors.fill: parent
        cursorShape: Qt.PointingHandCursor
        onClicked: root.shouldBlur ? (root.revealed = true) : root.opened()
    }

    ColumnLayout {
        anchors.fill: parent
        anchors.margins: 8
        spacing: 0

        RoundedImage {
            id: thumb
            Layout.fillWidth: true
            Layout.preferredHeight: width
            radius: Theme.shapeSmall
            maskColor: root.color
            source: root.firstFile ? (root.firstFile.thumbUrl || "") : ""
            hideImage: root.shouldBlur

            AppIcon {
                anchors.centerIn: parent
                visible: !root.firstFile
                name: "image"
                iconSize: 32
                color: Theme.outline
            }

            // Blur stand-in: scrim + label until tapped.
            Rectangle {
                anchors.fill: parent
                radius: thumb.radius
                color: Theme.surfaceContainerHighest
                opacity: root.shouldBlur ? 1 : 0
                visible: opacity > 0
                Behavior on opacity { NumberAnimation { duration: Theme.durMedium } }
                Column {
                    anchors.centerIn: parent
                    spacing: 6
                    AppIcon { anchors.horizontalCenter: parent.horizontalCenter; name: "shield"; iconSize: 24; color: Theme.onSurfaceVariantColor }
                    Text { anchors.horizontalCenter: parent.horizontalCenter; text: "Tap to reveal"; color: Theme.onSurfaceVariantColor; font.pixelSize: Theme.labelMedium; font.weight: Font.Medium }
                }
            }

            Row {
                anchors.top: parent.top
                anchors.left: parent.left
                anchors.margins: 8
                spacing: 4
                Repeater {
                    model: [ { show: post.sticky, icon: "pin" }, { show: post.closed, icon: "lock" } ]
                    delegate: Rectangle {
                        visible: modelData.show === true
                        width: 24; height: 24; radius: 12
                        color: Theme.tertiaryContainer
                        AppIcon { anchors.centerIn: parent; name: modelData.icon; iconSize: 14; color: Theme.onTertiaryContainerColor }
                    }
                }
            }

            AppButton {
                anchors.top: parent.top
                anchors.right: parent.right
                anchors.margins: 4
                compact: true
                variant: root.bookmarked ? "filledIcon" : "tonalIcon"
                onMedia: !root.bookmarked
                iconName: root.bookmarked ? "bookmark-fill" : "bookmark"
                tooltip: root.bookmarked ? "Remove bookmark" : "Bookmark"
                opacity: root.bookmarked || cardHover.hovered ? 1 : 0
                visible: opacity > 0
                Behavior on opacity { NumberAnimation { duration: Theme.durShort } }
                onClicked: BookmarksController.toggle({
                    siteId: root.site.id, boardCode: NavigationController.boardCode,
                    threadId: String(post.threadId), subject: post.subject,
                    thumbUrl: root.firstFile ? root.firstFile.thumbUrl : "", replyCount: post.replyCount,
                })
            }
        }

        Text {
            Layout.fillWidth: true
            Layout.topMargin: 10
            Layout.leftMargin: 4
            Layout.rightMargin: 4
            text: post.subject || ("Thread " + post.threadId)
            color: Theme.onSurfaceColor
            font.pixelSize: Theme.titleSmall
            font.weight: Font.Medium
            elide: Text.ElideRight
            maximumLineCount: 1
        }

        Text {
            Layout.fillWidth: true
            Layout.topMargin: 4
            Layout.leftMargin: 4
            Layout.rightMargin: 4
            visible: !root.compact
            text: Theme.plainText(post.commentHtml)
            color: Theme.onSurfaceVariantColor
            font.pixelSize: Theme.bodySmall
            lineHeight: 1.15
            wrapMode: Text.WordWrap
            maximumLineCount: 3
            elide: Text.ElideRight
        }

        Item { Layout.fillHeight: true }

        PostStats {
            Layout.leftMargin: 4
            Layout.bottomMargin: 2
            replies: post.replyCount
            images: post.imageCount
        }
    }

    StateLayer {
        hovered: cardHover.hovered
        layerColor: Theme.onSurfaceColor
        z: 10
    }
}
