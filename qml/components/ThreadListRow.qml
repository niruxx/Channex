import QtQuick
import QtQuick.Layouts
import Channex

// "list" catalog mode row as an M3 list item inside a filled card:
// rounded 72px thumbnail, subject (titleSmall), two-line excerpt, and
// counts; hover state layer.
Rectangle {
    id: root
    required property var post
    property var site: ({})
    height: 96
    radius: Theme.shapeMedium
    color: Theme.surfaceContainerLow

    signal opened()

    // post itself has no thumbUrl/spoiler - those live on its first file.
    readonly property var firstFile: (post.files && post.files.length > 0) ? post.files[0] : null
    readonly property bool shouldBlur: (site.nsfw || (firstFile && firstFile.spoiler)) && SettingsManager.blurNsfw && !revealed
    property bool revealed: false

    HoverHandler { id: rowHover }

    RowLayout {
        anchors.fill: parent
        anchors.margins: 12
        spacing: 16

        RoundedImage {
            Layout.preferredWidth: 72
            Layout.preferredHeight: 72
            radius: Theme.shapeSmall
            maskColor: root.color
            source: root.firstFile ? (root.firstFile.thumbUrl || "") : ""
            hideImage: root.shouldBlur

            AppIcon {
                anchors.centerIn: parent
                visible: root.shouldBlur || !root.firstFile
                name: root.shouldBlur ? "shield" : "image"
                iconSize: 22
                color: Theme.onSurfaceVariantColor
            }
        }

        ColumnLayout {
            Layout.fillWidth: true
            spacing: 4

            RowLayout {
                Layout.fillWidth: true
                spacing: 6
                AppIcon { visible: post.sticky === true; name: "pin"; iconSize: 14; color: Theme.tertiary }
                AppIcon { visible: post.closed === true; name: "lock"; iconSize: 14; color: Theme.tertiary }
                Text {
                    Layout.fillWidth: true
                    text: post.subject || ("Thread " + post.threadId)
                    color: Theme.onSurfaceColor
                    font.pixelSize: Theme.titleSmall
                    font.weight: Font.Medium
                    elide: Text.ElideRight
                }
            }
            Text {
                Layout.fillWidth: true
                text: Theme.plainText(post.commentHtml)
                color: Theme.onSurfaceVariantColor
                font.pixelSize: Theme.bodyMedium
                elide: Text.ElideRight
                maximumLineCount: 1
                wrapMode: Text.WordWrap
            }
            PostStats { replies: post.replyCount; images: post.imageCount }
        }
    }

    StateLayer { hovered: rowHover.hovered; pressed: rowMouse.pressed; z: 10 }

    MouseArea {
        id: rowMouse
        anchors.fill: parent
        z: 11
        cursorShape: Qt.PointingHandCursor
        onClicked: root.shouldBlur ? (root.revealed = true) : root.opened()
    }
}
