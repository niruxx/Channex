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

    Rectangle {
        anchors.fill: parent
        color: "#000000"
        opacity: 0.9
        MouseArea {
            anchors.fill: parent
            onClicked: LightboxController.close()
        }
    }

    // Subtle top/bottom vignette for a touch more cinematic depth than
    // a single flat backdrop.
    Rectangle {
        anchors.fill: parent
        gradient: Gradient {
            GradientStop { position: 0.0; color: "#00000055" }
            GradientStop { position: 0.5; color: "transparent" }
            GradientStop { position: 1.0; color: "#00000055" }
        }
    }

    // Image branch
    Item {
        anchors.fill: parent
        visible: !root.isVideo
        clip: true

        BusyIndicator {
            anchors.centerIn: parent
            running: LightboxController.loadState === "loading"
            visible: running
        }

        Column {
            anchors.centerIn: parent
            visible: LightboxController.loadState === "error"
            spacing: 10
            Text { text: "Couldn't load this file."; color: "white"; anchors.horizontalCenter: parent.horizontalCenter }
            Row {
                spacing: 8
                anchors.horizontalCenter: parent.horizontalCenter
                AppButton { text: "Retry"; onClicked: LightboxController.open(LightboxController.files, LightboxController.index) }
                AppButton { text: "Open in browser"; onClicked: ExternalReply.openUrl(root.file.url || "") }
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
            anchors.margins: 10
            height: 44
            radius: Theme.radiusMd
            color: "#000000b0"

            RowLayout {
                anchors.fill: parent
                anchors.leftMargin: 10
                anchors.rightMargin: 10
                spacing: 10

                AppButton {
                    flat: true
                    iconName: player.playbackState === MediaPlayer.PlayingState ? "pause" : "play"
                    onClicked: player.playbackState === MediaPlayer.PlayingState ? player.pause() : player.play()
                }

                Text {
                    text: root.formatTime(player.position)
                    color: "white"
                    font.pixelSize: 11
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

                    background: Rectangle {
                        x: seekSlider.leftPadding
                        y: seekSlider.topPadding + seekSlider.availableHeight / 2 - height / 2
                        width: seekSlider.availableWidth
                        height: 4
                        radius: 2
                        color: "#ffffff33"
                        Rectangle {
                            width: seekSlider.visualPosition * parent.width
                            height: parent.height
                            radius: 2
                            color: Theme.accent
                        }
                    }
                    handle: Rectangle {
                        x: seekSlider.leftPadding + seekSlider.visualPosition * (seekSlider.availableWidth - width)
                        y: seekSlider.topPadding + seekSlider.availableHeight / 2 - height / 2
                        width: 12; height: 12; radius: 6
                        color: Theme.accent
                    }
                }

                Text {
                    text: root.formatTime(player.duration)
                    color: "white"
                    font.pixelSize: 11
                }

                AppButton {
                    flat: true
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
        iconName: "chevron-left"
        visible: LightboxController.files.length > 1
        anchors.left: parent.left
        anchors.verticalCenter: parent.verticalCenter
        anchors.leftMargin: 16
        onClicked: LightboxController.prev()
    }
    AppButton {
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
            radius: Theme.radiusSm
            color: Theme.surface3
            border.width: current ? 2 : 0
            border.color: Theme.accent
            clip: true
            scale: current ? 1.0 : 0.92
            opacity: current ? 1.0 : 0.6

            Behavior on scale { NumberAnimation { duration: 140; easing.type: Easing.OutCubic } }
            Behavior on opacity { NumberAnimation { duration: 140 } }
            Behavior on border.color { ColorAnimation { duration: 140 } }

            Image {
                anchors.fill: parent
                anchors.margins: thumbDelegate.current ? 0 : 2
                source: modelData.thumbUrl || ""
                fillMode: Image.PreserveAspectCrop
                asynchronous: true
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
            text: "Queue"
            iconName: "plus"
            onClicked: DownloadManager.startJob("lightbox", SettingsManager.downloadDir.length ? SettingsManager.downloadDir : ".",
                [{ url: root.file.url || "", fileName: root.file.name || "file" }])
        }
        AppButton {
            text: "Save to disk"
            iconName: "download"
            checked: true
            onClicked: DownloadManager.startJob("lightbox-save", SettingsManager.downloadDir.length ? SettingsManager.downloadDir : ".",
                [{ url: root.file.url || "", fileName: root.file.name || "file" }])
        }
        AppButton {
            iconName: "x"
            flat: true
            onClicked: LightboxController.close()
        }
    }

    Keys.onEscapePressed: LightboxController.close()
    Keys.onLeftPressed: LightboxController.prev()
    Keys.onRightPressed: LightboxController.next()
    focus: visible
}
