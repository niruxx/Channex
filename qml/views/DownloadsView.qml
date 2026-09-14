import QtQuick
import QtQuick.Controls.Basic
import QtQuick.Layouts
import Channex
import "../components"

// Mirrors src/components/downloads/DownloadsPanel.tsx: session-only
// list of DownloadManager jobs, each with a per-file status list.
Item {
    ListView {
        anchors.fill: parent
        anchors.margins: 12
        spacing: 10
        clip: true
        model: DownloadManager.jobs

        populate: Transition {
            NumberAnimation { properties: "opacity"; from: 0; to: 1; duration: 200; easing.type: Easing.OutCubic }
        }
        add: Transition {
            NumberAnimation { properties: "opacity"; from: 0; to: 1; duration: 180; easing.type: Easing.OutCubic }
        }
        // No `displaced` transition: each job's delegate height is
        // variable (grows with its file count), and animating y against
        // a still-settling variable height is what caused the thread
        // reply list's overlap bug - see ThreadView.qml.

        Text {
            anchors.centerIn: parent
            visible: DownloadManager.jobs.length === 0
            text: "No downloads this session."
            color: Theme.inkFaint
        }

        delegate: Rectangle {
            width: ListView.view.width
            height: header.height + itemsColumn.height + 24
            radius: Theme.radiusMd
            color: Theme.surface2
            border.color: Theme.border

            Column {
                anchors.fill: parent
                anchors.margins: 12
                spacing: 8

                RowLayout {
                    id: header
                    width: parent.width
                    Text { text: modelData.label; color: Theme.ink; font.bold: true; Layout.fillWidth: true }
                    Text {
                        readonly property int done: modelData.items.filter(function (i) { return i.status === "done" }).length
                        text: done + " / " + modelData.items.length
                        color: Theme.inkFaint
                        font.pixelSize: 11
                    }
                    AppButton { text: "Open folder"; flat: true; onClicked: DownloadManager.openDirectory(modelData.destDir) }
                }

                Column {
                    id: itemsColumn
                    width: parent.width
                    spacing: 3
                    Repeater {
                        model: modelData.items
                        delegate: RowLayout {
                            width: itemsColumn.width
                            Text {
                                Layout.fillWidth: true
                                text: modelData.fileName
                                color: Theme.inkDim
                                font.pixelSize: 11
                                elide: Text.ElideMiddle
                            }
                            Text {
                                text: modelData.status
                                color: modelData.status === "error" ? Theme.danger
                                     : modelData.status === "done" ? Theme.success : Theme.inkFaint
                                font.pixelSize: 11
                            }
                        }
                    }
                }
            }
        }
    }
}
