import QtQuick
import QtQuick.Controls.Basic
import QtQuick.Layouts
import Channex

// Opened from ThreadView's "Download" button instead of immediately
// queuing every file: a grid of square, tap-to-toggle tiles (all
// selected by default) so the user can exclude specific files before
// the job is queued.
Dialog {
    id: root
    modal: true
    parent: Overlay.overlay
    x: Math.round((parent.width - width) / 2)
    y: Math.round((parent.height - height) / 2)
    width: Math.min(640, parent ? parent.width - 80 : 640)
    height: Math.min(640, parent ? parent.height - 80 : 640)
    title: "Download files"
    padding: 0

    property var files: []
    property string jobLabel: ""
    property string destDir: ""
    property var selectedMap: ({})

    readonly property int selectedCount: files.filter(function (f) { return root.selectedMap[f.url] === true }).length

    function openWithFiles(fileList, label, dir) {
        var sel = {}
        for (var i = 0; i < fileList.length; i++)
            sel[fileList[i].url] = true
        root.selectedMap = sel
        root.files = fileList
        root.jobLabel = label
        root.destDir = dir
        open()
    }

    function setAll(value) {
        var sel = {}
        for (var i = 0; i < root.files.length; i++)
            sel[root.files[i].url] = value
        root.selectedMap = sel
    }

    function toggle(url) {
        var sel = root.selectedMap
        sel[url] = !sel[url]
        root.selectedMap = sel
    }

    enter: Transition {
        NumberAnimation { properties: "opacity"; from: 0; to: 1; duration: 160; easing.type: Easing.OutCubic }
        NumberAnimation { properties: "scale"; from: 0.92; to: 1; duration: 160; easing.type: Easing.OutCubic }
    }
    exit: Transition {
        NumberAnimation { properties: "opacity"; from: 1; to: 0; duration: 120; easing.type: Easing.InCubic }
        NumberAnimation { properties: "scale"; from: 1; to: 0.92; duration: 120; easing.type: Easing.InCubic }
    }

    Overlay.modal: Rectangle {
        color: "#00000099"
        Behavior on opacity { NumberAnimation { duration: 160 } }
    }

    background: Rectangle { color: Theme.surface; radius: Theme.radiusLg; border.color: Theme.border }

    header: ColumnLayout {
        Layout.fillWidth: true
        spacing: 10

        RowLayout {
            Layout.fillWidth: true
            Layout.margins: 20
            Layout.bottomMargin: 0
            Label { text: root.title; color: Theme.ink; font.bold: true; font.pixelSize: 16; Layout.fillWidth: true }
            AppButton { text: "Select all"; flat: true; onClicked: root.setAll(true) }
            AppButton { text: "Select none"; flat: true; onClicked: root.setAll(false) }
        }

        Rectangle { Layout.fillWidth: true; height: 1; color: Theme.borderSoft }
    }

    contentItem: GridView {
        id: grid
        clip: true
        cellWidth: 128
        cellHeight: 128
        leftMargin: 16
        rightMargin: 16
        topMargin: 16
        bottomMargin: 16
        model: root.files

        populate: Transition {
            NumberAnimation { properties: "opacity"; from: 0; to: 1; duration: 180; easing.type: Easing.OutCubic }
            NumberAnimation { properties: "scale"; from: 0.9; to: 1; duration: 180; easing.type: Easing.OutCubic }
        }

        delegate: Item {
            width: grid.cellWidth
            height: grid.cellHeight

            readonly property bool selected: root.selectedMap[modelData.url] === true

            Rectangle {
                anchors.fill: parent
                anchors.margins: 6
                radius: Theme.radiusMd
                color: Theme.surface3
                clip: true
                border.width: selected ? 3 : 1
                border.color: selected ? Theme.accent : Theme.borderSoft

                Behavior on border.color { ColorAnimation { duration: 120 } }

                Image {
                    anchors.fill: parent
                    anchors.margins: parent.border.width
                    source: modelData.thumbUrl || ""
                    fillMode: Image.PreserveAspectCrop
                    asynchronous: true
                    opacity: selected ? 1.0 : 0.45

                    Behavior on opacity { NumberAnimation { duration: 120 } }
                }

                AppIcon {
                    visible: modelData.isVideo === true
                    anchors.centerIn: parent
                    name: "play"
                    iconSize: 18
                    color: "white"
                }

                Rectangle {
                    anchors.top: parent.top
                    anchors.right: parent.right
                    anchors.margins: 6
                    width: 22; height: 22; radius: 11
                    color: selected ? Theme.accent : "#00000099"
                    border.width: selected ? 0 : 1
                    border.color: "#ffffff88"

                    AppIcon {
                        visible: selected
                        anchors.centerIn: parent
                        name: "check"
                        iconSize: 12
                        color: "#0c0e11"
                    }
                }

                Rectangle {
                    anchors.bottom: parent.bottom
                    anchors.left: parent.left
                    anchors.right: parent.right
                    height: 18
                    visible: modelData.size > 0
                    color: "#00000099"
                    Text {
                        anchors.centerIn: parent
                        text: modelData.size ? Math.round(modelData.size / 1024) + " KB" : ""
                        color: "white"
                        font.pixelSize: 9
                    }
                }

                MouseArea {
                    anchors.fill: parent
                    cursorShape: Qt.PointingHandCursor
                    onClicked: root.toggle(modelData.url)
                }
            }
        }
    }

    footer: ColumnLayout {
        Layout.fillWidth: true
        spacing: 0

        Rectangle { Layout.fillWidth: true; height: 1; color: Theme.borderSoft }

        RowLayout {
            Layout.fillWidth: true
            Layout.margins: 16
            Text {
                Layout.fillWidth: true
                text: root.selectedCount + " of " + root.files.length + " selected"
                color: Theme.inkFaint
                font.pixelSize: 12
            }
            AppButton { text: "Cancel"; flat: true; onClicked: root.close() }
            AppButton {
                text: "Download selected"
                iconName: "download"
                checked: true
                enabled: root.selectedCount > 0
                onClicked: {
                    var toDownload = []
                    for (var i = 0; i < root.files.length; i++) {
                        var f = root.files[i]
                        if (root.selectedMap[f.url] === true)
                            toDownload.push({ url: f.url, fileName: f.name })
                    }
                    DownloadManager.startJob(root.jobLabel, root.destDir, toDownload)
                    root.close()
                }
            }
        }
    }
}
