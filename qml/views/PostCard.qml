import QtQuick
import QtQuick.Controls.Basic
import QtQuick.Layouts
import Channex
import "../components"

// One post in a thread, as an M3 card. Quote links are encoded by
// CommentFormatter as href="quote:<id>"; QML's Text.onLinkActivated
// handles them (see CommentFormatter.h), and Theme.themeComment()
// re-colors the formatter's inline quote/greentext/spoiler colors to
// the current scheme.
//
// Density/decoration follows SettingsManager.replyDisplayStyle
// ("compact" | "standard" | "modern"):
//   compact  - tight padding, small thumbs, no file captions
//   standard - the default card
//   modern   - roomier, larger corners, poster avatar, primary stripe
// The OP is always set on a higher surface tier than replies.
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

    readonly property int pad: isCompact ? 12 : (isModern ? 20 : 16)
    readonly property int rowSpacing: isCompact ? 6 : (isModern ? 12 : 10)
    readonly property int thumbSize: isCompact ? 112 : (isModern ? 176 : 160)
    readonly property int cardRadius: isCompact ? Theme.shapeSmall : (isModern ? Theme.shapeLarge : Theme.shapeMedium)
    readonly property int commentSize: isCompact ? Theme.bodySmall + 1 : Theme.bodyMedium

    readonly property string posterName: (post.name && post.name.length) ? post.name : "Anonymous"

    radius: cardRadius
    color: post.isOp ? Theme.surfaceContainer : Theme.surfaceContainerLow
    implicitHeight: contentColumn.implicitHeight + pad * 2

    // Right-click anywhere on the card to report the post - 4chan has
    // no public report API (by design, to prevent abuse/automation),
    // so like ExternalReply's reply handoff, this opens 4chan's own
    // report page in the system browser where the user picks a reason
    // and solves the captcha themselves. Declared low in the stacking
    // order so left-clicks still reach the links/thumbnails above it.
    MouseArea {
        anchors.fill: parent
        acceptedButtons: Qt.RightButton
        onClicked: if (root.site.id === "4chan") postContextMenu.popup()
    }

    AppMenu {
        id: postContextMenu
        AppMenuItem {
            text: "Report post to 4chan…"
            iconName: "alert"
            onTriggered: ExternalReply.openUrl(
                "https://sys.4chan.org/" + NavigationController.boardCode + "/imgboard.php?mode=report&no=" + root.postId)
        }
    }

    // Modern-only primary stripe down the left edge, inset so it never
    // pokes past the rounded corners.
    Rectangle {
        visible: root.isModern
        anchors.left: parent.left
        anchors.top: parent.top
        anchors.bottom: parent.bottom
        anchors.topMargin: root.cardRadius
        anchors.bottomMargin: root.cardRadius
        width: 4
        radius: 2
        color: post.isOp ? Theme.primary : Theme.withAlpha(Theme.primary, 0.5)
    }

    ColumnLayout {
        id: contentColumn
        anchors.fill: parent
        anchors.margins: root.pad
        spacing: root.rowSpacing

        RowLayout {
            Layout.fillWidth: true
            spacing: root.isCompact ? 6 : 8

            // Modern-only poster avatar.
            Rectangle {
                visible: root.isModern
                Layout.preferredWidth: 32
                Layout.preferredHeight: 32
                radius: 16
                color: post.isOp ? Theme.primaryContainer : Theme.secondaryContainer
                Text {
                    anchors.centerIn: parent
                    text: root.posterName.charAt(0).toUpperCase()
                    color: post.isOp ? Theme.onPrimaryContainerColor : Theme.onSecondaryContainerColor
                    font.pixelSize: Theme.titleSmall
                    font.weight: Font.Medium
                }
            }

            Rectangle {
                visible: post.isOp === true && !root.isModern
                implicitWidth: opLabel.implicitWidth + 12; implicitHeight: 20; radius: 10
                color: Theme.primaryContainer
                Text { id: opLabel; anchors.centerIn: parent; text: "OP"; color: Theme.onPrimaryContainerColor; font.pixelSize: Theme.labelSmall; font.weight: Font.Bold }
            }

            Text {
                visible: post.subject && post.subject.length
                text: post.subject || ""
                color: Theme.onSurfaceColor
                font.pixelSize: root.isCompact ? Theme.titleSmall - 1 : Theme.titleSmall
                font.weight: Font.Bold
                Layout.maximumWidth: root.width * 0.45
                elide: Text.ElideRight
            }
            Text {
                text: root.posterName
                color: Theme.primary
                font.pixelSize: root.isCompact ? Theme.labelMedium : Theme.labelLarge
                font.weight: Font.Medium
            }
            Text {
                visible: post.tripcode && post.tripcode.length
                text: post.tripcode || ""
                color: Theme.tertiary
                font.pixelSize: Theme.labelMedium
            }
            Rectangle {
                visible: post.capcode && post.capcode.length
                color: Theme.tertiaryContainer
                radius: Theme.shapeSmall
                implicitWidth: capcodeText.implicitWidth + 12; implicitHeight: 20
                Text { id: capcodeText; anchors.centerIn: parent; text: (post.capcode || "").toUpperCase(); color: Theme.onTertiaryContainerColor; font.pixelSize: Theme.labelSmall; font.weight: Font.Bold }
            }
            AppIcon { visible: post.sticky === true; name: "pin"; iconSize: 14; color: Theme.tertiary }
            AppIcon { visible: post.closed === true; name: "lock"; iconSize: 14; color: Theme.tertiary }
            Item { Layout.fillWidth: true }
            Text {
                text: "No." + root.postId
                color: noHover.hovered ? Theme.primary : Theme.outline
                font.pixelSize: Theme.labelMedium
                font.weight: Font.Medium
                HoverHandler { id: noHover }
                MouseArea { anchors.fill: parent; cursorShape: Qt.PointingHandCursor; onClicked: root.quoteClicked(root.postId) }
            }
        }

        Text {
            Layout.fillWidth: true
            visible: text.length > 0
            textFormat: Text.RichText
            wrapMode: Text.WordWrap
            text: Theme.themeComment(post.commentHtml)
            color: Theme.onSurfaceColor
            linkColor: Theme.quoteLink
            font.pixelSize: root.commentSize
            lineHeight: 1.2
            onLinkActivated: (link) => {
                if (link.indexOf("quote:") === 0) root.quoteClicked(link.substring(6))
                else Qt.openUrlExternally(link)
            }
            HoverHandler { cursorShape: parent.hoveredLink.length > 0 ? Qt.PointingHandCursor : Qt.ArrowCursor }
        }

        Flow {
            Layout.fillWidth: true
            spacing: root.isCompact ? 8 : 10
            visible: post.files && post.files.length > 0

            Repeater {
                model: post.files || []
                delegate: Column {
                    id: fileDelegate
                    property bool revealed: false
                    readonly property bool shouldBlur: (root.site.nsfw || modelData.spoiler) && SettingsManager.blurNsfw && !revealed
                    spacing: 6

                    RoundedImage {
                        id: fileThumb
                        width: root.thumbSize; height: root.thumbSize
                        radius: root.isModern ? Theme.shapeMedium : Theme.shapeSmall
                        maskColor: root.color
                        source: modelData.thumbUrl || ""
                        hideImage: fileDelegate.shouldBlur

                        HoverHandler { id: thumbHover }

                        Rectangle {
                            anchors.fill: parent
                            radius: fileThumb.radius
                            color: Theme.surfaceContainerHighest
                            opacity: fileDelegate.shouldBlur ? 1 : 0
                            visible: opacity > 0
                            Behavior on opacity { NumberAnimation { duration: Theme.durMedium } }
                            Column {
                                anchors.centerIn: parent
                                spacing: 6
                                AppIcon { anchors.horizontalCenter: parent.horizontalCenter; name: "shield"; iconSize: 22; color: Theme.onSurfaceVariantColor }
                                Text { anchors.horizontalCenter: parent.horizontalCenter; text: "Tap to reveal"; color: Theme.onSurfaceVariantColor; font.pixelSize: Theme.labelSmall; font.weight: Font.Medium }
                            }
                        }

                        AppIcon {
                            visible: modelData.isVideo === true && !fileDelegate.shouldBlur
                            anchors.centerIn: parent
                            name: "play"
                            iconSize: 28
                            color: "#ffffff"
                        }

                        MouseArea {
                            anchors.fill: parent
                            cursorShape: Qt.PointingHandCursor
                            onClicked: {
                                if (fileDelegate.shouldBlur) { fileDelegate.revealed = true; return }
                                root.fileOpenRequested(modelData.url)
                            }
                        }

                        AppButton {
                            anchors.top: parent.top
                            anchors.right: parent.right
                            anchors.margins: 6
                            compact: true
                            variant: "tonalIcon"
                            onMedia: true
                            iconName: "download"
                            tooltip: "Download"
                            opacity: !fileDelegate.shouldBlur && thumbHover.hovered ? 1 : 0
                            visible: opacity > 0
                            Behavior on opacity { NumberAnimation { duration: Theme.durShort } }
                            onClicked: {
                                var destBase = SettingsManager.downloadDir.length ? SettingsManager.downloadDir : "."
                                DownloadManager.startJob(modelData.name, destBase, [{ url: modelData.url, fileName: modelData.name }])
                            }
                        }
                    }

                    Text {
                        width: root.thumbSize
                        visible: !root.isCompact
                        text: modelData.name + (modelData.size ? " · " + Math.round(modelData.size / 1024) + " KB" : "")
                        color: Theme.onSurfaceVariantColor
                        font.pixelSize: Theme.labelSmall
                        elide: Text.ElideMiddle
                    }
                }
            }
        }

        // Backlinks as small outlined chips.
        Flow {
            Layout.fillWidth: true
            visible: root.backlinks.length > 0
            spacing: 6
            Repeater {
                model: root.backlinks
                delegate: Rectangle {
                    color: "transparent"
                    radius: Theme.shapeSmall
                    border.width: 1
                    border.color: Theme.outlineVariant
                    width: label.implicitWidth + 16; height: 24
                    StateLayer { layerColor: Theme.quoteLink; hovered: blHover.hovered; pressed: blMouse.pressed }
                    HoverHandler { id: blHover }
                    Text { id: label; anchors.centerIn: parent; text: ">>" + modelData; color: Theme.quoteLink; font.pixelSize: Theme.labelMedium; font.weight: Font.Medium }
                    MouseArea { id: blMouse; anchors.fill: parent; cursorShape: Qt.PointingHandCursor; onClicked: root.quoteClicked(modelData) }
                }
            }
        }
    }
}
