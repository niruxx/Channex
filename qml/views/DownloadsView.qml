import QtQuick
import QtQuick.Controls.Basic
import QtQuick.Layouts
import Channex
import "../components"

// Session download jobs. Each job is a compact M3 card - status icon,
// label, "N of M files" summary, linear progress, and actions - backed
// by DownloadManager.jobs (a list model whose rows update in place).
// The per-file list is only built while a job's "Show files" preview is
// open, and is virtualized (ListView), so a several-hundred-file job
// costs nothing to display until you ask to see its files.
Item {
    id: root

    ColumnLayout {
        anchors.fill: parent
        spacing: 0

        RowLayout {
            Layout.fillWidth: true
            Layout.leftMargin: 32
            Layout.rightMargin: 24
            Layout.topMargin: 28
            Layout.bottomMargin: 16
            spacing: 12

            ColumnLayout {
                Layout.fillWidth: true
                spacing: 4
                Text { text: "Downloads"; color: Theme.onSurfaceColor; font.pixelSize: Theme.headlineMedium }
                Text {
                    text: DownloadManager.jobCount === 0 ? "This session's download jobs"
                        : (DownloadManager.activeJobCount > 0 ? DownloadManager.activeJobCount + " in progress · " : "")
                          + DownloadManager.jobCount + (DownloadManager.jobCount === 1 ? " job" : " jobs")
                          + " · clearing the list keeps your files"
                    color: Theme.onSurfaceVariantColor
                    font.pixelSize: Theme.bodyLarge
                }
            }

            Item { Layout.fillWidth: true }

            // Clears the history list only - downloaded files stay on
            // disk, and jobs still in progress stay listed.
            AppButton {
                variant: "tonal"
                text: "Clear history"
                iconName: "x"
                tooltip: "Remove finished jobs from this list. Downloaded files are not deleted."
                visible: DownloadManager.jobCount > DownloadManager.activeJobCount
                onClicked: DownloadManager.clearFinished()
            }
        }

        ListView {
            id: list
            Layout.fillWidth: true
            Layout.fillHeight: true
            leftMargin: 24
            rightMargin: 24
            bottomMargin: 24
            spacing: 12
            clip: true
            model: DownloadManager.jobs
            ScrollBar.vertical: AppScrollBar {}

            add: Transition {
                NumberAnimation { properties: "opacity"; from: 0; to: 1; duration: Theme.durMedium; easing.type: Easing.BezierSpline; easing.bezierCurve: Theme.easeStandard }
            }
            remove: Transition {
                NumberAnimation { properties: "opacity"; to: 0; duration: Theme.durShort }
            }
            // No `displaced` transition: delegate height changes when a
            // file preview expands, and animating y against a settling
            // variable height causes overlap (see ThreadView.qml).

            EmptyState {
                anchors.centerIn: parent
                visible: DownloadManager.jobCount === 0
                iconName: "download"
                text: "No downloads yet"
                supportingText: "Files you save from threads or the lightbox show up here."
            }

            delegate: Rectangle {
                id: job
                required property string jobId
                required property string label
                required property string destDir
                required property int total
                required property int done
                required property int failed
                required property int active
                required property bool finished

                width: ListView.view.width - 48
                height: jobColumn.implicitHeight + 32
                radius: Theme.shapeLarge
                color: Theme.surfaceContainerLow

                readonly property real progress: total > 0 ? (done + failed) / total : 1
                property bool expanded: false
                property var files: []

                function refreshFiles() { files = expanded ? DownloadManager.jobFiles(jobId) : [] }
                onExpandedChanged: refreshFiles()

                // jobFilesChanged is already batched in C++; only an open
                // preview listens, and it re-reads at most this often.
                Connections {
                    target: DownloadManager
                    enabled: job.expanded
                    function onJobFilesChanged(id) { if (id === job.jobId) refreshThrottle.restart() }
                }
                Timer { id: refreshThrottle; interval: 120; onTriggered: job.refreshFiles() }

                ColumnLayout {
                    id: jobColumn
                    x: 20; y: 16
                    width: parent.width - 40
                    spacing: 12

                    RowLayout {
                        Layout.fillWidth: true
                        spacing: 16

                        Rectangle {
                            implicitWidth: 40; implicitHeight: 40; radius: 20
                            color: job.failed > 0 && job.finished ? Theme.errorContainer
                                 : (job.finished ? Theme.primaryContainer : Theme.secondaryContainer)
                            AppIcon {
                                anchors.centerIn: parent
                                name: job.failed > 0 && job.finished ? "alert" : (job.finished ? "check" : "download")
                                iconSize: 20
                                color: job.failed > 0 && job.finished ? Theme.onErrorContainerColor
                                     : (job.finished ? Theme.onPrimaryContainerColor : Theme.onSecondaryContainerColor)
                            }
                        }

                        ColumnLayout {
                            Layout.fillWidth: true
                            spacing: 2
                            Text {
                                Layout.fillWidth: true
                                text: job.label
                                color: Theme.onSurfaceColor
                                font.pixelSize: Theme.bodyLarge
                                font.weight: Font.Medium
                                elide: Text.ElideMiddle
                            }
                            Text {
                                Layout.fillWidth: true
                                text: (job.finished ? (job.failed > 0 ? "Finished · " : "Complete · ") : "Downloading · ")
                                      + job.done + " of " + job.total + (job.total === 1 ? " file" : " files")
                                      + (job.failed > 0 ? " · " + job.failed + " failed" : "")
                                color: job.failed > 0 ? Theme.error : Theme.onSurfaceVariantColor
                                font.pixelSize: Theme.bodyMedium
                                elide: Text.ElideRight
                            }
                        }

                        AppButton {
                            visible: job.failed > 0 && job.finished
                            variant: "tonal"
                            text: "Retry failed"
                            iconName: "refresh"
                            onClicked: DownloadManager.retryFailed(job.jobId)
                        }
                        AppButton {
                            variant: "icon"
                            iconName: "folder"
                            tooltip: "Open folder"
                            onClicked: DownloadManager.openDirectory(job.destDir)
                        }
                        AppButton {
                            visible: job.finished
                            variant: "icon"
                            iconName: "x"
                            tooltip: "Remove from list (files are kept)"
                            onClicked: DownloadManager.removeJob(job.jobId)
                        }
                    }

                    // M3 linear progress indicator (determinate): active
                    // track, gap, inactive track, stop dot at the end.
                    Item {
                        Layout.fillWidth: true
                        implicitHeight: 4
                        Rectangle {
                            id: activeTrack
                            height: 4; radius: 2
                            width: Math.max(job.progress > 0 ? 4 : 0, parent.width * job.progress - (job.finished ? 0 : 2))
                            color: job.failed > 0 && job.finished ? Theme.error : Theme.primary
                            Behavior on width { NumberAnimation { duration: Theme.durMedium; easing.type: Easing.BezierSpline; easing.bezierCurve: Theme.easeStandard } }
                        }
                        Rectangle {
                            visible: !job.finished
                            x: activeTrack.width + (job.progress > 0 ? 4 : 0)
                            width: Math.max(0, parent.width - x)
                            height: 4; radius: 2
                            color: Theme.secondaryContainer
                        }
                        Rectangle {
                            visible: !job.finished
                            anchors.right: parent.right
                            width: 4; height: 4; radius: 2
                            color: Theme.primary
                        }
                    }

                    // Preview toggle - the only way per-file rows get built.
                    AppButton {
                        visible: job.total > 0
                        Layout.leftMargin: -12
                        variant: "text"
                        compact: true
                        text: job.expanded ? "Hide files" : "Show " + job.total + (job.total === 1 ? " file" : " files")
                        iconName: "chevron-down"
                        onClicked: job.expanded = !job.expanded
                    }

                    Rectangle {
                        Layout.fillWidth: true
                        visible: job.expanded
                        implicitHeight: Math.min(fileList.contentHeight, 288) + 8
                        radius: Theme.shapeMedium
                        color: Theme.surfaceContainer

                        ListView {
                            id: fileList
                            anchors.fill: parent
                            anchors.margins: 4
                            clip: true
                            model: job.files
                            boundsBehavior: Flickable.StopAtBounds
                            ScrollBar.vertical: AppScrollBar {}

                            delegate: Item {
                                width: fileList.width
                                height: 36

                                readonly property string status: modelData.status

                                RowLayout {
                                    anchors.fill: parent
                                    anchors.leftMargin: 12
                                    anchors.rightMargin: 12
                                    spacing: 12

                                    AppIcon {
                                        name: status === "done" ? "check" : (status === "error" ? "alert" : (status === "downloading" ? "download" : "image"))
                                        iconSize: 16
                                        color: status === "done" ? Theme.success
                                             : status === "error" ? Theme.error
                                             : status === "downloading" ? Theme.primary : Theme.outline
                                    }
                                    Text {
                                        Layout.fillWidth: true
                                        text: modelData.fileName
                                        color: status === "pending" ? Theme.onSurfaceVariantColor : Theme.onSurfaceColor
                                        font.pixelSize: Theme.bodyMedium
                                        elide: Text.ElideMiddle
                                    }
                                    Text {
                                        text: status === "error" && modelData.error ? modelData.error : status
                                        Layout.maximumWidth: 220
                                        elide: Text.ElideRight
                                        color: status === "error" ? Theme.error
                                             : status === "done" ? Theme.success : Theme.onSurfaceVariantColor
                                        font.pixelSize: Theme.labelMedium
                                        font.weight: Font.Medium
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}
