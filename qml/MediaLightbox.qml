import QtQuick
import QtQuick.Controls.Basic
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

    Rectangle {
        anchors.fill: parent
        color: "#000000"
        opacity: 0.88
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
            visible: LightboxController.loadState === "loaded"
            asynchronous: true
            fillMode: Image.PreserveAspectFit
            width: Math.min(parent.width - 80, implicitWidth) * (LightboxController.zoomed ? 1.9 : 1)
            height: Math.min(parent.height - 160, implicitHeight) * (LightboxController.zoomed ? 1.9 : 1)

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
        anchors.fill: parent
        anchors.margins: 40
        visible: root.isVideo

        MediaPlayer {
            id: player
            source: root.isVideo ? (root.file.url || "") : ""
            videoOutput: videoOutput
            loops: MediaPlayer.Infinite
            audioOutput: AudioOutput { muted: SettingsManager.muteWebmsByDefault; volume: 1.0 }
            onSourceChanged: if (root.isVideo) play()
        }

        VideoOutput {
            id: videoOutput
            anchors.fill: parent
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
        text: "‹"
        font.pixelSize: 18
        visible: LightboxController.files.length > 1
        anchors.left: parent.left
        anchors.verticalCenter: parent.verticalCenter
        anchors.leftMargin: 16
        onClicked: LightboxController.prev()
    }
    AppButton {
        text: "›"
        font.pixelSize: 18
        visible: LightboxController.files.length > 1
        anchors.right: parent.right
        anchors.verticalCenter: parent.verticalCenter
        anchors.rightMargin: 16
        onClicked: LightboxController.next()
    }

    // Dot indicator
    Row {
        anchors.bottom: parent.bottom
        anchors.horizontalCenter: parent.horizontalCenter
        anchors.bottomMargin: 84
        spacing: 6
        visible: LightboxController.files.length > 1
        Repeater {
            model: LightboxController.files.length
            delegate: Rectangle {
                width: 6; height: 6; radius: 3
                color: index === LightboxController.index ? Theme.accent : "#666"
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
            text: "＋ Queue"
            onClicked: DownloadManager.startJob("lightbox", SettingsManager.downloadDir.length ? SettingsManager.downloadDir : ".",
                [{ url: root.file.url || "", fileName: root.file.name || "file" }])
        }
        AppButton {
            text: "Save to disk"
            onClicked: DownloadManager.startJob("lightbox-save", SettingsManager.downloadDir.length ? SettingsManager.downloadDir : ".",
                [{ url: root.file.url || "", fileName: root.file.name || "file" }])
        }
        AppButton {
            text: "Close"
            checked: true
            onClicked: LightboxController.close()
        }
    }

    Keys.onEscapePressed: LightboxController.close()
    Keys.onLeftPressed: LightboxController.prev()
    Keys.onRightPressed: LightboxController.next()
    focus: visible
}
