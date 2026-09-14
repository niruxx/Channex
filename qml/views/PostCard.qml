import QtQuick
import QtQuick.Controls.Basic
import QtQuick.Layouts
import Channex

// Mirrors src/components/thread/PostCard.tsx + CommentText.tsx. Quote
// links are encoded by CommentFormatter as href="quote:<id>"; QML's
// native Text.onLinkActivated replaces the DOM data-quotelink click
// interception the Tauri app needs (see CommentFormatter.h).
//
// Visual density/decoration is driven entirely by
// SettingsManager.replyDisplayStyle ("compact" | "standard" |
// "modern") - "standard" intentionally reproduces the original,
// pre-setting look byte-for-byte so existing users see no change
// unless they opt into something else.
Rectangle {
    id: root
    required property var post
    property var backlinks: []
    property var site: ({})
    property string postId: post.id !== undefined ? String(post.id) : ""

    signal quoteClicked(string postId)
    signal fileOpenRequested(string url)

    readonly property string style: SettingsManager.replyDisplayStyle
    readonly property bool isCompact: style === "compact"
    readonly property bool isModern: style === "modern"

    readonly property int pad: isCompact ? 8 : (isModern ? 16 : 12)
    readonly property int rowSpacing: isCompact ? 5 : (isModern ? 10 : 8)
    readonly property int thumbSize: isCompact ? 110 : (isModern ? 168 : 160)
    readonly property int cardRadius: isCompact ? Theme.radiusSm : (isModern ? Theme.radiusLg : Theme.radiusMd)
    readonly property int subjectSize: isCompact ? 12 : (isModern ? 14 : 13)
    readonly property int commentSize: isCompact ? 12 : 13
    readonly property int metaSize: isCompact ? 10 : 11
    readonly property int tripcodeSize: isCompact ? 10 : (isModern ? 11 : 12)

    readonly property string posterInitial: ((post.name && post.name.length) ? post.name : "Anonymous").charAt(0).toUpperCase()

    radius: cardRadius
    color: post.isOp ? (isModern ? Qt.tint(Theme.surface2, Qt.rgba(Theme.accent.r, Theme.accent.g, Theme.accent.b, 0.06)) : Theme.surface2) : Theme.surface
    border.color: Theme.border
    implicitHeight: contentColumn.implicitHeight + pad * 2

    property var revealedFiles: ({})

    // Modern-only accent stripe down the left edge, inset from the top/
    // bottom so it never pokes past the card's rounded corners.
    Rectangle {
        visible: root.isModern
        anchors.left: parent.left
        anchors.top: parent.top
        anchors.bottom: parent.bottom
        anchors.margins: root.cardRadius * 0.6
        width: 3
        radius: 1.5
        color: Theme.accent
        opacity: 0.7
    }

    ColumnLayout {
        id: contentColumn
        anchors.fill: parent
        anchors.margins: root.pad
        anchors.leftMargin: root.isModern ? root.pad + 8 : root.pad
        spacing: root.rowSpacing

        RowLayout {
            Layout.fillWidth: true
            spacing: root.isCompact ? 4 : 6

            // Modern-only small avatar - gives anonymous posts a visual
            // anchor, matching the accent-circle language TitleBar.qml
            // already uses for the site avatar.
            Rectangle {
                visible: root.isModern
                Layout.preferredWidth: 22
                Layout.preferredHeight: 22
                radius: 11
                color: Theme.accent
                Text {
                    anchors.centerIn: parent
                    text: root.posterInitial
                    color: "#0c0e11"
                    font.pixelSize: 10
                    font.bold: true
                }
            }

            Text { visible: post.subject && post.subject.length; text: post.subject; color: Theme.ink; font.bold: true; font.pixelSize: root.subjectSize }
            Text { visible: post.name && post.name.length; text: post.name; color: "#6fd39a"; font.pixelSize: root.subjectSize }
            Text { visible: post.tripcode && post.tripcode.length; text: post.tripcode; color: "#b39ce8"; font.pixelSize: root.tripcodeSize }
            Rectangle {
                visible: post.capcode && post.capcode.length
                color: Theme.accent; radius: 4
                width: capcodeText.implicitWidth + 10; height: 18
                Text { id: capcodeText; anchors.centerIn: parent; text: (post.capcode || "").toUpperCase(); color: "#0c0e11"; font.pixelSize: 10; font.bold: true }
            }
            Text { visible: post.sticky; text: "📌"; font.pixelSize: 11 }
            Text { visible: post.closed; text: "🔒"; font.pixelSize: 11 }
            Item { Layout.fillWidth: true }
            Text {
                text: "No." + root.postId
                color: Theme.inkFaint
                font.pixelSize: root.metaSize
                MouseArea { anchors.fill: parent; cursorShape: Qt.PointingHandCursor; onClicked: root.quoteClicked(root.postId) }
            }
        }

        Text {
            Layout.fillWidth: true
            textFormat: Text.RichText
            wrapMode: Text.WordWrap
            text: post.commentHtml || ""
            color: Theme.ink
            font.pixelSize: root.commentSize
            onLinkActivated: (link) => {
                if (link.indexOf("quote:") === 0) root.quoteClicked(link.substring(6))
                else Qt.openUrlExternally(link)
            }
        }

        Flow {
            Layout.fillWidth: true
            visible: root.backlinks.length > 0
            spacing: 4
            Repeater {
                model: root.backlinks
                delegate: Rectangle {
                    color: Theme.surface3; radius: 4
                    width: label.implicitWidth + 12; height: 20
                    Text { id: label; anchors.centerIn: parent; text: ">>" + modelData; color: "#7fb0e0"; font.pixelSize: 11 }
                    MouseArea { anchors.fill: parent; cursorShape: Qt.PointingHandCursor; onClicked: root.quoteClicked(modelData) }
                }
            }
        }

        Flow {
            Layout.fillWidth: true
            spacing: root.isCompact ? 6 : 8
            visible: post.files && post.files.length > 0

            Repeater {
                model: post.files || []
                delegate: Column {
                    property bool revealed: false
                    readonly property bool shouldBlur: (root.site.nsfw || modelData.spoiler) && SettingsManager.blurNsfw && !revealed
                    spacing: 4

                    Rectangle {
                        width: root.thumbSize; height: root.thumbSize
                        radius: root.isModern ? Theme.radiusMd : Theme.radiusSm
                        color: Theme.surface3
                        clip: true

                        Image {
                            anchors.fill: parent
                            source: modelData.thumbUrl || ""
                            fillMode: Image.PreserveAspectCrop
                            asynchronous: true
                            visible: !shouldBlur
                        }
                        Rectangle {
                            anchors.fill: parent
                            color: "black"
                            opacity: shouldBlur ? 0.85 : 0
                            visible: opacity > 0
                            Behavior on opacity { NumberAnimation { duration: 180; easing.type: Easing.OutCubic } }
                            Text { anchors.centerIn: parent; text: "Tap to reveal"; color: "white"; font.pixelSize: 10 }
                        }

                        Rectangle {
                            anchors.top: parent.top; anchors.right: parent.right; anchors.margins: 4
                            width: 22; height: 22; radius: 5
                            color: "#00000099"
                            opacity: shouldBlur ? 0 : 1
                            visible: opacity > 0
                            Behavior on opacity { NumberAnimation { duration: 180; easing.type: Easing.OutCubic } }
                            Text { anchors.centerIn: parent; text: "⇩"; color: "white"; font.pixelSize: 12 }
                            MouseArea {
                                anchors.fill: parent
                                onClicked: {
                                    var destBase = SettingsManager.downloadDir.length ? SettingsManager.downloadDir : "."
                                    DownloadManager.startJob(modelData.name, destBase, [{ url: modelData.url, fileName: modelData.name }])
                                }
                            }
                        }

                        MouseArea {
                            anchors.fill: parent
                            cursorShape: Qt.PointingHandCursor
                            onClicked: {
                                if (shouldBlur) { revealed = true; return }
                                root.fileOpenRequested(modelData.url)
                            }
                        }
                    }

                    Text {
                        width: root.thumbSize
                        visible: !root.isCompact
                        text: modelData.name + (modelData.size ? " (" + Math.round(modelData.size / 1024) + " KB)" : "")
                        color: Theme.inkFaint
                        font.pixelSize: 10
                        elide: Text.ElideMiddle
                    }
                }
            }
        }
    }
}
