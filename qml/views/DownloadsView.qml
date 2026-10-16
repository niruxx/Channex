import QtQuick
import QtQuick.Controls.Basic
import QtQuick.Layouts
import Channex
import "../components"

// Session-only list of DownloadManager jobs: one M3 card per job with
// a linear progress indicator and a per-file status list.
Item {
    ColumnLayout {
        anchors.fill: parent
        spacing: 0

        ColumnLayout {
            Layout.fillWidth: true
            Layout.leftMargin: 32
            Layout.topMargin: 28
            Layout.bottomMargin: 16
            spacing: 4
            Text { text: "Downloads"; color: Theme.onSurfaceColor; font.pixelSize: Theme.headlineMedium }
            Text { text: "This session's download jobs"; color: Theme.onSurfaceVariantColor; font.pixelSize: Theme.bodyLarge }
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

            populate: Transition {
                NumberAnimation { properties: "opacity"; from: 0; to: 1; duration: Theme.durMedium }
            }
            add: Transition {
                NumberAnimation { properties: "opacity"; from: 0; to: 1; duration: Theme.durMedium }
            }
            // No `displaced` transition: each job's delegate height is
            // variable (grows with its file count), and animating y against
            // a still-settling variable height is what caused the thread
            // reply list's overlap bug - see ThreadView.qml.

            EmptyState {
                anchors.centerIn: parent
                visible: DownloadManager.jobs.length === 0
                iconName: "download"
                text: "No downloads yet"
                supportingText: "Files you save from threads or the lightbox show up here."
            }

            delegate: Rectangle {
                id: job
                width: list.width - 48
                height: jobColumn.implicitHeight + 32
                radius: Theme.shapeLarge
                color: Theme.surfaceContainerLow

                readonly property int doneCount: modelData.items.filter(function (i) { return i.status === "done" }).length
                readonly property int errorCount: modelData.items.filter(function (i) { return i.status === "error" }).length
                readonly property real progress: modelData.items.length > 0 ? (doneCount + errorCount) / modelData.items.length : 0
                readonly property bool finished: progress >= 1

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
                            color: job.errorCount > 0 ? Theme.errorContainer : (job.finished ? Theme.primaryContainer : Theme.secondaryContainer)
                            AppIcon {
                                anchors.centerIn: parent
                                name: job.errorCount > 0 ? "alert" : (job.finished ? "check" : "download")
                                iconSize: 20
                                color: job.errorCount > 0 ? Theme.onErrorContainerColor : (job.finished ? Theme.onPrimaryContainerColor : Theme.onSecondaryContainerColor)
                            }
                        }

                        ColumnLayout {
                            Layout.fillWidth: true
                            spacing: 2
                            Text { Layout.fillWidth: true; text: modelData.label; color: Theme.onSurfaceColor; font.pixelSize: Theme.bodyLarge; font.weight: Font.Medium; elide: Text.ElideMiddle }
                            Text {
                                text: job.doneCount + " of " + modelData.items.length + " files" + (job.errorCount > 0 ? " · " + job.errorCount + " failed" : "")
                                color: job.errorCount > 0 ? Theme.error : Theme.onSurfaceVariantColor
                                font.pixelSize: Theme.bodyMedium
                            }
                        }

                        AppButton { variant: "text"; text: "Open folder"; iconName: "folder"; onClicked: DownloadManager.openDirectory(modelData.destDir) }
                    }

                    // M3 linear progress indicator (determinate): active
                    // track, 4px gap, inactive track, stop dot at the end.
                    Item {
                        Layout.fillWidth: true
                        implicitHeight: 4
                        Rectangle {
                            id: activeTrack
                            height: 4; radius: 2
                            width: Math.max(job.progress > 0 ? 4 : 0, parent.width * job.progress - (job.finished ? 0 : 2))
                            color: Theme.primary
                            Behavior on width { NumberAnimation { duration: Theme.durMedium; easing.type: Easing.BezierSpline; easing.bezierCurve: Theme.easeStandard } }
                        }
                        Rectangle {
                            visible: !job.finished
                            x: activeTrack.width + (job.progress > 0 ? 4 : 0)
                            width: parent.width - x
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

                    Column {
                        id: itemsColumn
                        Layout.fillWidth: true
                        spacing: 2
                        Repeater {
                            model: modelData.items
                            delegate: RowLayout {
                                width: itemsColumn.width
                                spacing: 10
                                Text {
                                    Layout.fillWidth: true
                                    text: modelData.fileName
                                    color: Theme.onSurfaceVariantColor
                                    font.pixelSize: Theme.bodySmall
                                    elide: Text.ElideMiddle
                                }
                                Rectangle {
                                    implicitHeight: 20
                                    implicitWidth: statusText.implicitWidth + 12
                                    radius: Theme.shapeSmall
                                    color: modelData.status === "error" ? Theme.errorContainer
                                         : modelData.status === "done" ? Theme.withAlpha(Theme.success, 0.16) : Theme.surfaceContainerHighest
                                    Text {
                                        id: statusText
                                        anchors.centerIn: parent
                                        text: modelData.status
                                        color: modelData.status === "error" ? Theme.onErrorContainerColor
                                             : modelData.status === "done" ? Theme.success : Theme.onSurfaceVariantColor
                                        font.pixelSize: Theme.labelSmall
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
