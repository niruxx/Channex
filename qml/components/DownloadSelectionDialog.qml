import QtQuick
import QtQuick.Controls.Basic
import QtQuick.Layouts
import Channex

// Opened from ThreadView's "Download" button instead of immediately
// queuing every file: a grid of square, tap-to-toggle tiles (all
// selected by default) so the user can exclude specific files before
// the job is queued. M3 full-height dialog with dividers around the
// scrolling grid.
AppDialog {
    id: root
    width: Math.min(640, parent ? parent.width - 80 : 640)
    height: Math.min(640, parent ? parent.height - 80 : 640)
    title: "Download files"
    padding: 0
    topPadding: 0

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

    header: ColumnLayout {
        spacing: 16

        RowLayout {
            Layout.fillWidth: true
            Layout.leftMargin: 24
            Layout.rightMargin: 16
            Layout.topMargin: 24
            spacing: 4
            Text { text: root.title; color: Theme.onSurfaceColor; font.pixelSize: Theme.headlineSmall; Layout.fillWidth: true }
            AppButton { variant: "text"; text: "All"; onClicked: root.setAll(true) }
            AppButton { variant: "text"; text: "None"; onClicked: root.setAll(false) }
        }

        Rectangle { Layout.fillWidth: true; height: 1; color: Theme.outlineVariant }
    }

    contentItem: GridView {
        id: grid
        clip: true
        cellWidth: Math.floor((width - 32) / Math.max(1, Math.floor((width - 32) / 132)))
        cellHeight: cellWidth
        leftMargin: 16
        rightMargin: 16
        topMargin: 12
        bottomMargin: 12
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
                id: tile
                anchors.fill: parent
                anchors.margins: 4
                radius: Theme.shapeMedium
                color: selected ? Theme.secondaryContainer : Theme.surfaceContainerHigh
                Behavior on color { ColorAnimation { duration: Theme.durShort } }

                // Selected tiles inset the thumbnail (the M3 "selected
                // item shrinks into a tonal frame" pattern).
                RoundedImage {
                    anchors.fill: parent
                    anchors.margins: selected ? 8 : 0
                    radius: selected ? Theme.shapeSmall : Theme.shapeMedium
                    maskColor: tile.color
                    source: modelData.thumbUrl || ""
                    opacity: selected ? 1.0 : 0.55
                    Behavior on anchors.margins { NumberAnimation { duration: Theme.durShort; easing.type: Easing.BezierSpline; easing.bezierCurve: Theme.easeStandard } }
                    Behavior on opacity { NumberAnimation { duration: Theme.durShort } }

                    AppIcon {
                        visible: modelData.isVideo === true
                        anchors.centerIn: parent
                        name: "play"
                        iconSize: 22
                        color: "white"
                    }

                    Rectangle {
                        anchors.bottom: parent.bottom
                        anchors.left: parent.left
                        anchors.margins: 6
                        height: 18
                        width: sizeText.implicitWidth + 10
                        radius: 9
                        visible: modelData.size > 0
                        color: Qt.rgba(0, 0, 0, 0.6)
                        Text {
                            id: sizeText
                            anchors.centerIn: parent
                            text: modelData.size ? Math.round(modelData.size / 1024) + " KB" : ""
                            color: "white"
                            font.pixelSize: Theme.labelSmall
                        }
                    }
                }

                Rectangle {
                    anchors.top: parent.top
                    anchors.right: parent.right
                    anchors.margins: 4
                    width: 24; height: 24; radius: 12
                    color: selected ? Theme.primary : Qt.rgba(0, 0, 0, 0.45)
                    border.width: selected ? 0 : 2
                    border.color: "#ffffffcc"

                    AppIcon {
                        visible: selected
                        anchors.centerIn: parent
                        name: "check"
                        iconSize: 14
                        color: Theme.onPrimaryColor
                    }
                }

                HoverHandler { id: tileHover }
                StateLayer { hovered: tileHover.hovered; layerColor: Theme.onSurfaceColor }

                MouseArea {
                    anchors.fill: parent
                    cursorShape: Qt.PointingHandCursor
                    onClicked: root.toggle(modelData.url)
                }
            }
        }
    }

    footer: ColumnLayout {
        spacing: 0

        Rectangle { Layout.fillWidth: true; height: 1; color: Theme.outlineVariant }

        RowLayout {
            Layout.fillWidth: true
            Layout.margins: 16
            Layout.leftMargin: 24
            spacing: 8
            Text {
                Layout.fillWidth: true
                text: root.selectedCount + " of " + root.files.length + " selected"
                color: Theme.onSurfaceVariantColor
                font.pixelSize: Theme.bodyMedium
            }
            AppButton { variant: "text"; text: "Cancel"; onClicked: root.close() }
            AppButton {
                variant: "filled"
                text: "Download " + root.selectedCount
                iconName: "download"
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
