import QtQuick
import QtQuick.Controls.Basic
import QtQuick.Layouts
import Channex

// Mirrors src/components/thread/PostCard.tsx + CommentText.tsx. Quote
// links are encoded by CommentFormatter as href="quote:<id>"; QML's
// native Text.onLinkActivated replaces the DOM data-quotelink click
// interception the Tauri app needs (see CommentFormatter.h).
Rectangle {
    id: root
    required property var post
    property var backlinks: []
    property var site: ({})
    property string postId: post.id !== undefined ? String(post.id) : ""

    signal quoteClicked(string postId)
    signal fileOpenRequested(string url)

    radius: Theme.radiusMd
    color: post.isOp ? Theme.surface2 : Theme.surface
    border.color: Theme.border
    implicitHeight: contentColumn.implicitHeight + 24

    property var revealedFiles: ({})

    ColumnLayout {
        id: contentColumn
        anchors.fill: parent
        anchors.margins: 12
        spacing: 8

        RowLayout {
            Layout.fillWidth: true
            spacing: 6
            Text { visible: post.subject && post.subject.length; text: post.subject; color: Theme.ink; font.bold: true; font.pixelSize: 13 }
            Text { visible: post.name && post.name.length; text: post.name; color: "#6fd39a"; font.pixelSize: 13 }
            Text { visible: post.tripcode && post.tripcode.length; text: post.tripcode; color: "#b39ce8"; font.pixelSize: 12 }
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
                font.pixelSize: 11
                MouseArea { anchors.fill: parent; cursorShape: Qt.PointingHandCursor; onClicked: root.quoteClicked(root.postId) }
            }
        }

        Text {
            Layout.fillWidth: true
            textFormat: Text.RichText
            wrapMode: Text.WordWrap
            text: post.commentHtml || ""
            color: Theme.ink
            font.pixelSize: 13
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
            spacing: 8
            visible: post.files && post.files.length > 0

            Repeater {
                model: post.files || []
                delegate: Column {
                    property bool revealed: false
                    readonly property bool shouldBlur: (root.site.nsfw || modelData.spoiler) && SettingsManager.blurNsfw && !revealed
                    spacing: 4

                    Rectangle {
                        width: 160; height: 160; radius: Theme.radiusSm
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
                            visible: !shouldBlur
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
                        width: 160
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
