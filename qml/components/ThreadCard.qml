import QtQuick
import QtQuick.Controls.Basic
import Channex

// Mirrors src/components/catalog/ThreadCard.tsx: square thumbnail +
// subject/excerpt + reply/image counts, sticky/closed badges, an
// NSFW/spoiler blur-until-tapped overlay, and a bookmark toggle.
Rectangle {
    id: root
    required property var post
    property bool compact: false
    property var site: ({})

    radius: Theme.radiusMd
    color: cardHover.hovered ? Theme.surface3 : Theme.surface2
    border.color: cardHover.hovered ? Theme.border : Theme.borderSoft

    Behavior on color { ColorAnimation { duration: 120; easing.type: Easing.OutCubic } }
    Behavior on border.color { ColorAnimation { duration: 120; easing.type: Easing.OutCubic } }

    HoverHandler { id: cardHover }

    // post itself has no thumbUrl/spoiler - those live on its first file.
    readonly property var firstFile: (post.files && post.files.length > 0) ? post.files[0] : null
    readonly property bool shouldBlur: (site.nsfw || (firstFile && firstFile.spoiler)) && SettingsManager.blurNsfw && !revealed
    property bool revealed: false

    // Reference bookmarks.length so this re-evaluates on bookmarksChanged
    // (isBookmarked() alone is a plain invokable call QML won't re-bind on).
    readonly property bool bookmarked: BookmarksController.bookmarks.length >= 0
        && BookmarksController.isBookmarked(site.id, NavigationController.boardCode, String(post.threadId))

    signal opened()

    Column {
        anchors.fill: parent
        anchors.margins: 8
        spacing: 6

        Rectangle {
            width: parent.width
            height: width
            radius: Theme.radiusSm
            color: Theme.surface3
            clip: true

            Image {
                anchors.fill: parent
                source: root.firstFile ? (root.firstFile.thumbUrl || "") : ""
                fillMode: Image.PreserveAspectCrop
                asynchronous: true
                visible: !root.shouldBlur
            }

            Rectangle {
                anchors.fill: parent
                color: "black"
                opacity: 0.85
                visible: root.shouldBlur
                Text {
                    anchors.centerIn: parent
                    text: "Tap to reveal"
                    color: "white"
                    font.pixelSize: 11
                }
            }

            Row {
                anchors.top: parent.top
                anchors.left: parent.left
                anchors.margins: 4
                spacing: 4
                Rectangle {
                    visible: post.sticky
                    color: "#00000099"; radius: 4; width: 18; height: 18
                    Text { anchors.centerIn: parent; text: "📌"; font.pixelSize: 10 }
                }
                Rectangle {
                    visible: post.closed
                    color: "#00000099"; radius: 4; width: 18; height: 18
                    Text { anchors.centerIn: parent; text: "🔒"; font.pixelSize: 10 }
                }
            }

            Rectangle {
                anchors.top: parent.top
                anchors.right: parent.right
                anchors.margins: 4
                radius: 9
                color: root.bookmarked ? Theme.accent : "#00000099"
                width: 22; height: 22

                Behavior on color { ColorAnimation { duration: 150; easing.type: Easing.OutCubic } }

                Text { anchors.centerIn: parent; text: "★"; font.pixelSize: 11; color: root.bookmarked ? "#0c0e11" : "white" }
                MouseArea {
                    anchors.fill: parent
                    onClicked: BookmarksController.toggle({
                        siteId: root.site.id, boardCode: NavigationController.boardCode,
                        threadId: String(post.threadId), subject: post.subject,
                        thumbUrl: root.firstFile ? root.firstFile.thumbUrl : "", replyCount: post.replyCount,
                    })
                }
            }
        }

        Text {
            width: parent.width
            text: post.subject || ("Thread #" + post.threadId)
            color: Theme.ink
            font.pixelSize: 13
            font.bold: true
            elide: Text.ElideRight
            maximumLineCount: 1
        }

        Text {
            width: parent.width
            visible: !root.compact
            text: post.commentHtml ? post.commentHtml.replace(/<[^>]*>/g, "") : ""
            color: Theme.inkDim
            font.pixelSize: 12
            wrapMode: Text.WordWrap
            maximumLineCount: 3
            elide: Text.ElideRight
        }

        Row {
            spacing: 10
            Text { text: "💬 " + post.replyCount; color: Theme.inkFaint; font.pixelSize: 11 }
            Text { text: "🖼 " + post.imageCount; color: Theme.inkFaint; font.pixelSize: 11 }
        }
    }

    MouseArea {
        anchors.fill: parent
        z: -1
        cursorShape: Qt.PointingHandCursor
        onClicked: root.shouldBlur ? (root.revealed = true) : root.opened()
    }
}
