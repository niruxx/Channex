import QtQuick
import QtQuick.Controls.Basic
import QtQuick.Layouts
import QtMultimedia
import Channex
import "components"

// Mirrors src/components/lightbox/MediaLightbox.tsx. Mounted once at
// the app root (see Main.qml) and driven entirely by LightboxController.
// Images load through image://lightbox/... (LightboxController fetches
// bytes itself, see that class's doc comment); videos play directly
// from their remote URL via QtMultimedia, autoplay-muted by default.
Item {
    id: root
    anchors.fill: parent
    visible: opacity > 0
    opacity: LightboxController.isOpen ? 1 : 0
    z: 1000

    Behavior on opacity { NumberAnimation { duration: 150; easing.type: Easing.OutCubic } }

    readonly property var file: LightboxController.currentFile
    readonly property bool isVideo: file.isVideo === true
    property real dragStartX: 0

    function formatTime(ms) {
        if (!ms || ms <= 0 || isNaN(ms)) return "0:00"
        const totalSeconds = Math.floor(ms / 1000)
        const minutes = Math.floor(totalSeconds / 60)
        const seconds = totalSeconds % 60
        return minutes + ":" + (seconds < 10 ? "0" : "") + seconds
    }

    // Near-opaque media backdrop (M3 media viewers sit on black, not
    // a translucent scrim). Kept solid so RoundedImage corner masks in
    // the filmstrip below can match it.
    Rectangle {
        anchors.fill: parent
        color: "#0a0a0a"
        opacity: 0.97
        MouseArea {
            anchors.fill: parent
            onClicked: LightboxController.close()
        }
    }

    // Image branch
    Item {
        anchors.fill: parent
        visible: !root.isVideo
        clip: true

        AppBusyIndicator {
            anchors.centerIn: parent
            color: Theme.inversePrimary
            running: LightboxController.loadState === "loading"
        }

        Column {
            anchors.centerIn: parent
            visible: LightboxController.loadState === "error"
            spacing: 16
            AppIcon { anchors.horizontalCenter: parent.horizontalCenter; name: "alert"; iconSize: 40; color: "#ffffff" }
            Text { text: "Couldn't load this file"; color: "white"; font.pixelSize: Theme.titleMedium; anchors.horizontalCenter: parent.horizontalCenter }
            Row {
                spacing: 8
                anchors.horizontalCenter: parent.horizontalCenter
                AppButton { variant: "filled"; onMedia: true; text: "Retry"; iconName: "refresh"; onClicked: LightboxController.open(LightboxController.files, LightboxController.index) }
                AppButton { variant: "outlined"; onMedia: true; text: "Open in browser"; iconName: "open-external"; onClicked: ExternalReply.openUrl(root.file.url || "") }
            }
        }

        Image {
            id: mainImage
            anchors.centerIn: parent
            source: LightboxController.imageSource
            opacity: LightboxController.loadState === "loaded" ? 1 : 0
            visible: opacity > 0
            asynchronous: true
            fillMode: Image.PreserveAspectFit
            width: Math.min(parent.width - 80, implicitWidth) * (LightboxController.zoomed ? 1.9 : 1)
            height: Math.min(parent.height - 160, implicitHeight) * (LightboxController.zoomed ? 1.9 : 1)

            // Crossfades between files instead of popping: loadState drops
            // back to "loading" on every prev/next/goTo (see
            // LightboxController::loadCurrent), so this fades the old image
            // out and the new one in around that state change for free.
            Behavior on opacity { NumberAnimation { duration: 160; easing.type: Easing.OutCubic } }
            Behavior on width { NumberAnimation { duration: 200; easing.type: Easing.OutCubic } }
            Behavior on height { NumberAnimation { duration: 200; easing.type: Easing.OutCubic } }

            MouseArea {
                anchors.fill: parent
                cursorShape: LightboxController.zoomed ? Qt.ZoomOutCursor : Qt.ZoomInCursor
                onClicked: LightboxController.zoomed = !LightboxController.zoomed
            }
        }
    }

    // Video branch
    Item {
        id: videoBranch
        anchors.fill: parent
        anchors.margins: 40
        visible: root.isVideo

        MediaPlayer {
            id: player
            source: root.isVideo ? (root.file.url || "") : ""
            videoOutput: videoOutput
            loops: MediaPlayer.Infinite
            audioOutput: AudioOutput { id: audioOutput; muted: SettingsManager.muteWebmsByDefault; volume: 1.0 }
            onSourceChanged: if (root.isVideo) play()
        }

        VideoOutput {
            id: videoOutput
            anchors.fill: parent
            opacity: (player.mediaStatus === MediaPlayer.Loading || player.mediaStatus === MediaPlayer.NoMedia) ? 0 : 1
            Behavior on opacity { NumberAnimation { duration: 160; easing.type: Easing.OutCubic } }

            MouseArea {
                anchors.fill: parent
                onClicked: player.playbackState === MediaPlayer.PlayingState ? player.pause() : player.play()
            }
        }

        Rectangle {
            id: videoControls
            anchors.left: parent.left
            anchors.right: parent.right
            anchors.bottom: parent.bottom
            anchors.margins: 12
            height: 56
            radius: 28
            color: Qt.rgba(0, 0, 0, 0.62)

            RowLayout {
                anchors.fill: parent
                anchors.leftMargin: 8
                anchors.rightMargin: 8
                spacing: 12

                AppButton {
                    variant: "filledIcon"
                    onMedia: true
                    iconName: player.playbackState === MediaPlayer.PlayingState ? "pause" : "play"
                    onClicked: player.playbackState === MediaPlayer.PlayingState ? player.pause() : player.play()
                }

                Text {
                    text: root.formatTime(player.position)
                    color: "white"
                    font.pixelSize: Theme.labelMedium
                    font.weight: Font.Medium
                }

                Slider {
                    id: seekSlider
                    Layout.fillWidth: true
                    from: 0
                    to: Math.max(player.duration, 1)
                    onPressedChanged: if (!pressed) player.setPosition(value)

                    Connections {
                        target: player
                        function onPositionChanged() {
                            if (!seekSlider.pressed) seekSlider.value = player.position
                        }
                    }

                    // M3 slider: 16px rounded active/inactive tracks split
                    // by a gap around a 4px vertical bar handle.
                    background: Item {
                        x: seekSlider.leftPadding
                        y: seekSlider.topPadding + seekSlider.availableHeight / 2 - height / 2
                        width: seekSlider.availableWidth
                        height: 12
                        readonly property real hx: seekSlider.visualPosition * (width - 4)
                        Rectangle {
                            width: Math.max(0, parent.hx - 6)
                            height: parent.height
                            radius: 6
                            topRightRadius: 2
                            bottomRightRadius: 2
                            color: Theme.inversePrimary
                        }
                        Rectangle {
                            x: parent.hx + 10
                            width: Math.max(0, parent.width - x)
                            height: parent.height
                            radius: 6
                            topLeftRadius: 2
                            bottomLeftRadius: 2
                            color: "#ffffff38"
                        }
                    }
                    handle: Rectangle {
                        x: seekSlider.leftPadding + seekSlider.visualPosition * (seekSlider.availableWidth - 4)
                        y: seekSlider.topPadding + seekSlider.availableHeight / 2 - height / 2
                        width: seekSlider.pressed ? 2 : 4
                        height: 32
                        radius: 2
                        color: Theme.inversePrimary
                    }
                }

                Text {
                    text: root.formatTime(player.duration)
                    color: "white"
                    font.pixelSize: Theme.labelMedium
                    font.weight: Font.Medium
                }

                AppButton {
                    variant: "icon"
                    onMedia: true
                    iconName: audioOutput.muted ? "volume-mute" : "volume"
                    onClicked: audioOutput.muted = !audioOutput.muted
                }
            }
        }
    }

    // Drag-to-swipe surface (image branch only, disabled while zoomed)
    MouseArea {
        anchors.fill: parent
        enabled: !root.isVideo && !LightboxController.zoomed && LightboxController.files.length > 1
        property real pressX: 0
        onPressed: (mouse) => pressX = mouse.x
        onReleased: (mouse) => {
            const delta = mouse.x - pressX
            if (delta > 80) LightboxController.prev()
            else if (delta < -80) LightboxController.next()
        }
    }

    // Nav arrows
    AppButton {
        variant: "tonalIcon"
        onMedia: true
        iconName: "chevron-left"
        visible: LightboxController.files.length > 1
        anchors.left: parent.left
        anchors.verticalCenter: parent.verticalCenter
        anchors.leftMargin: 16
        onClicked: LightboxController.prev()
    }
    AppButton {
        variant: "tonalIcon"
        onMedia: true
        iconName: "chevron-right"
        visible: LightboxController.files.length > 1
        anchors.right: parent.right
        anchors.verticalCenter: parent.verticalCenter
        anchors.rightMargin: 16
        onClicked: LightboxController.next()
    }

    // Thumbnail filmstrip - every other file in the thread, click to jump
    ListView {
        id: filmstrip
        anchors.bottom: parent.bottom
        anchors.horizontalCenter: parent.horizontalCenter
        anchors.bottomMargin: 84
        width: Math.min(parent.width - 40, contentWidth)
        height: 56
        orientation: ListView.Horizontal
        spacing: 8
        visible: LightboxController.files.length > 1
        model: LightboxController.files
        clip: true

        onCurrentIndexChanged: positionViewAtIndex(currentIndex, ListView.Contain)
        currentIndex: LightboxController.index

        delegate: Rectangle {
            id: thumbDelegate
            readonly property bool current: index === LightboxController.index
            width: 56; height: 56
            radius: Theme.shapeMedium
            color: "transparent"
            border.width: current ? 2 : 0
            border.color: Theme.inversePrimary
            scale: current ? 1.0 : 0.9
            opacity: current ? 1.0 : 0.55

            Behavior on scale { NumberAnimation { duration: 140; easing.type: Easing.OutCubic } }
            Behavior on opacity { NumberAnimation { duration: 140 } }
            Behavior on border.color { ColorAnimation { duration: 140 } }

            RoundedImage {
                anchors.fill: parent
                anchors.margins: thumbDelegate.current ? 4 : 0
                radius: thumbDelegate.current ? Theme.shapeSmall : Theme.shapeMedium
                maskColor: "#0a0a0a"
                placeholderColor: "#262626"
                source: modelData.thumbUrl || ""
            }

            AppIcon {
                visible: modelData.isVideo === true
                anchors.centerIn: parent
                name: "play"
                iconSize: 14
                color: "white"
                opacity: 0.9
            }

            MouseArea {
                anchors.fill: parent
                cursorShape: Qt.PointingHandCursor
                onClicked: LightboxController.goTo(index)
            }
        }
    }

    // Bottom action bar
    Row {
        anchors.bottom: parent.bottom
        anchors.right: parent.right
        anchors.margins: 20
        spacing: 8

        AppButton {
            variant: "tonal"
            onMedia: true
            text: "Queue"
            iconName: "plus"
            onClicked: DownloadManager.startJob("lightbox", SettingsManager.downloadDir.length ? SettingsManager.downloadDir : ".",
                [{ url: root.file.url || "", fileName: root.file.name || "file" }])
        }
        AppButton {
            text: "Save to disk"
            iconName: "download"
            variant: "filled"
            onMedia: true
            onClicked: DownloadManager.startJob("lightbox-save", SettingsManager.downloadDir.length ? SettingsManager.downloadDir : ".",
                [{ url: root.file.url || "", fileName: root.file.name || "file" }])
        }
        AppButton {
            iconName: "x"
            variant: "tonalIcon"
            onMedia: true
            tooltip: "Close"
            onClicked: LightboxController.close()
        }
    }

    Keys.onEscapePressed: LightboxController.close()
    Keys.onLeftPressed: LightboxController.prev()
    Keys.onRightPressed: LightboxController.next()
    focus: visible
}
