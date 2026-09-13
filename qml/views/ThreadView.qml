import QtQuick
import QtQuick.Controls.Basic
import QtQuick.Layouts
import Channex
import "../components"

// Mirrors src/components/thread/ThreadView.tsx: header bar (subject,
// post count, bookmark, download-all, open-in-browser, refresh),
// PostCard list, and the ReplyComposer.
Item {
    id: root
    readonly property var site: SitesController.findSite(NavigationController.siteId)

    ThreadController { id: controller }

    Connections {
        target: NavigationController
        function onFrameChanged() {
            if (NavigationController.view === "thread" && NavigationController.threadId)
                controller.load(NavigationController.siteId, NavigationController.boardCode, NavigationController.threadId)
        }
    }
    Connections {
        target: controller
        function onLoaded(replyCount) {
            BookmarksController.markSeen(NavigationController.siteId, NavigationController.boardCode, NavigationController.threadId, replyCount)
        }
    }

    Component.onCompleted: {
        if (NavigationController.threadId)
            controller.load(NavigationController.siteId, NavigationController.boardCode, NavigationController.threadId)
    }

    readonly property bool bookmarked: BookmarksController.bookmarks.length >= 0
        && BookmarksController.isBookmarked(NavigationController.siteId, NavigationController.boardCode, NavigationController.threadId)

    ColumnLayout {
        anchors.fill: parent
        spacing: 0

        RowLayout {
            Layout.fillWidth: true
            Layout.margins: 12
            spacing: 10

            Text {
                Layout.fillWidth: true
                text: (controller.subject.length ? controller.subject : ("Thread #" + NavigationController.threadId)) + "  ·  " + controller.postCount + " posts"
                color: Theme.ink
                font.bold: true
                font.pixelSize: 14
                elide: Text.ElideRight
            }

            AppButton {
                text: root.bookmarked ? "★ Bookmarked" : "☆ Bookmark"
                flat: true
                checked: root.bookmarked
                onClicked: BookmarksController.toggle({
                    siteId: NavigationController.siteId, boardCode: NavigationController.boardCode,
                    threadId: NavigationController.threadId, subject: controller.subject,
                    replyCount: controller.postCount - 1,
                })
            }

            AppButton {
                text: "Download all (" + controller.allFiles.length + ")"
                enabled: controller.allFiles.length > 0
                flat: true
                onClicked: {
                    var files = []
                    for (var i = 0; i < controller.allFiles.length; i++) {
                        var f = controller.allFiles[i]
                        files.push({ url: f.url, fileName: f.name })
                    }
                    var destBase = SettingsManager.downloadDir.length ? SettingsManager.downloadDir : "."
                    DownloadManager.startJob(
                        NavigationController.siteId + "-" + NavigationController.boardCode + "-" + NavigationController.threadId,
                        destBase + "/" + NavigationController.siteId + "-" + NavigationController.boardCode + "-" + NavigationController.threadId,
                        files)
                }
            }

            AppButton { text: "Open in browser"; flat: true; onClicked: ExternalReply.openUrl(controller.threadWebUrl) }
            AppButton { text: "⟳"; flat: true; onClicked: controller.reload() }
        }

        Rectangle { Layout.fillWidth: true; height: 1; color: Theme.borderSoft }

        ListView {
            id: postList
            Layout.fillWidth: true
            Layout.fillHeight: true
            clip: true
            model: controller
            spacing: 10
            leftMargin: 12; rightMargin: 12; topMargin: 12; bottomMargin: 12

            BusyIndicator { anchors.centerIn: parent; running: controller.loading; visible: controller.loading }

            delegate: Item {
                width: postList.width - 24
                height: postCard.implicitHeight

                // PostCard has a `required property`; using it directly as
                // the delegate root is unreliable with Qt Quick's implicit
                // role-to-required-property binding (post ends up
                // permanently undefined). Wrapping it in a plain Item root
                // and binding post/backlinks on the child avoids that.
                PostCard {
                    id: postCard
                    width: parent.width
                    post: model.post
                    backlinks: model.backlinks
                    site: root.site
                    onQuoteClicked: (postId) => {
                        var row = controller.rowForPostId(postId)
                        if (row >= 0) postList.positionViewAtIndex(row, ListView.Center)
                    }
                }
            }

            footer: ReplyComposer {
                width: postList.width - 24
                site: root.site
                threadWebUrl: controller.threadWebUrl
                onPosted: controller.reload()
            }
        }
    }
}
