import QtQuick
import Channex

// Mirrors src/components/catalog/ThreadListRow.tsx: single-line
// subject+excerpt with an 80x80 thumbnail, used by the "list" catalog
// view mode.
Rectangle {
    id: root
    required property var post
    property var site: ({})
    height: 88
    radius: Theme.radiusMd
    color: rowHover.hovered ? Theme.surface3 : Theme.surface2
    border.color: rowHover.hovered ? Theme.border : Theme.borderSoft

    Behavior on color { ColorAnimation { duration: 120; easing.type: Easing.OutCubic } }
    Behavior on border.color { ColorAnimation { duration: 120; easing.type: Easing.OutCubic } }

    HoverHandler { id: rowHover }

    signal opened()

    // post itself has no thumbUrl/spoiler - those live on its first file.
    readonly property var firstFile: (post.files && post.files.length > 0) ? post.files[0] : null
    readonly property bool shouldBlur: (site.nsfw || (firstFile && firstFile.spoiler)) && SettingsManager.blurNsfw && !revealed
    property bool revealed: false

    Row {
        anchors.fill: parent
        anchors.margins: 8
        spacing: 10

        Rectangle {
            width: 72; height: 72; radius: Theme.radiusSm
            color: Theme.surface3
            clip: true
            Image {
                anchors.fill: parent
                source: root.firstFile ? (root.firstFile.thumbUrl || "") : ""
                fillMode: Image.PreserveAspectCrop
                asynchronous: true
                visible: !root.shouldBlur
            }
            Rectangle { anchors.fill: parent; color: "black"; opacity: 0.85; visible: root.shouldBlur }
        }

        Column {
            width: parent.width - 90
            spacing: 4
            Text {
                width: parent.width
                text: post.subject || ("Thread #" + post.threadId)
                color: Theme.ink
                font.bold: true
                font.pixelSize: 13
                elide: Text.ElideRight
            }
            Text {
                width: parent.width
                text: post.commentHtml ? post.commentHtml.replace(/<[^>]*>/g, "") : ""
                color: Theme.inkDim
                font.pixelSize: 12
                elide: Text.ElideRight
                maximumLineCount: 2
                wrapMode: Text.WordWrap
            }
            Row {
                spacing: 10
                Text { text: "💬 " + post.replyCount; color: Theme.inkFaint; font.pixelSize: 11 }
                Text { text: "🖼 " + post.imageCount; color: Theme.inkFaint; font.pixelSize: 11 }
            }
        }
    }

    MouseArea {
        anchors.fill: parent
        cursorShape: Qt.PointingHandCursor
        onClicked: root.shouldBlur ? (root.revealed = true) : root.opened()
    }
}
