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

        // Header: titleLarge subject + supporting post/file counts,
        // then actions in descending M3 emphasis (tonal Download,
        // toggle-bookmark, then standard icon buttons).
        RowLayout {
            Layout.fillWidth: true
            Layout.leftMargin: 28
            Layout.rightMargin: 16
            Layout.topMargin: 18
            Layout.bottomMargin: 10
            spacing: 8

            ColumnLayout {
                Layout.fillWidth: true
                spacing: 2
                Text {
                    Layout.fillWidth: true
                    text: controller.subject.length ? controller.subject : ("Thread " + NavigationController.threadId)
                    color: Theme.onSurfaceColor
                    font.pixelSize: Theme.titleLarge
                    elide: Text.ElideRight
                }
                Text {
                    text: controller.postCount + " posts · " + controller.allFiles.length + " files"
                    color: Theme.onSurfaceVariantColor
                    font.pixelSize: Theme.bodyMedium
                }
            }

            AppButton {
                variant: "tonal"
                text: "Download"
                iconName: "download"
                enabled: controller.allFiles.length > 0
                onClicked: {
                    var destBase = SettingsManager.downloadDir.length ? SettingsManager.downloadDir : "."
                    var jobLabel = NavigationController.siteId + "-" + NavigationController.boardCode + "-" + NavigationController.threadId
                    downloadDialog.openWithFiles(controller.allFiles, jobLabel, destBase + "/" + jobLabel)
                }
            }

            AppButton {
                variant: "filledIcon"
                checkable: true
                checked: root.bookmarked
                iconName: root.bookmarked ? "bookmark-fill" : "bookmark"
                tooltip: root.bookmarked ? "Remove bookmark" : "Bookmark thread"
                onClicked: BookmarksController.toggle({
                    siteId: NavigationController.siteId, boardCode: NavigationController.boardCode,
                    threadId: NavigationController.threadId, subject: controller.subject,
                    replyCount: controller.postCount - 1,
                })
            }

            AppButton { variant: "icon"; iconName: "open-external"; tooltip: "Open in browser"; onClicked: ExternalReply.openUrl(controller.threadWebUrl) }
            AppButton { variant: "icon"; iconName: "refresh"; tooltip: "Refresh"; onClicked: controller.reload() }
        }

        DownloadSelectionDialog { id: downloadDialog }

        ListView {
            id: postList
            Layout.fillWidth: true
            Layout.fillHeight: true
            clip: true
            model: controller
            spacing: 8
            leftMargin: 16; rightMargin: 16; topMargin: 4; bottomMargin: 16
            ScrollBar.vertical: AppScrollBar {}

            populate: Transition {
                NumberAnimation { properties: "opacity"; from: 0; to: 1; duration: 200; easing.type: Easing.OutCubic }
            }
            add: Transition {
                NumberAnimation { properties: "opacity"; from: 0; to: 1; duration: 180; easing.type: Easing.OutCubic }
            }
            // Deliberately no `displaced` transition here: PostCard's
            // height is variable (depends on comment length/files), and
            // animating a ListView's displaced `y` against delegates
            // whose height can still be settling is a known Qt Quick
            // trap - the tween can lock items into a y that doesn't
            // match their actual (possibly since-changed) height,
            // leaving them visibly overlapping instead of stacked. Plain
            // opacity fades for populate/add carry the "proper
            // animation" intent without touching position.

            AppBusyIndicator { anchors.centerIn: parent; running: controller.loading && controller.postCount === 0 }

            delegate: Item {
                width: postList.width - 32
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
                    onFileOpenRequested: (url) => {
                        var all = controller.allFiles
                        var idx = 0
                        for (var i = 0; i < all.length; i++)
                            if (all[i].url === url) { idx = i; break }
                        LightboxController.open(all, idx)
                    }
                }
            }

            // ListView.spacing doesn't reliably apply between the last
            // delegate and the footer (it's a positioning quirk, not
            // something the footer item can fix from the inside), which
            // left the reply box sitting right up against the last
            // post - wrapping it and adding an explicit top margin
            // guarantees breathing room regardless of that.
            footer: Item {
                width: postList.width
                implicitHeight: replyComposer.implicitHeight + 48

                // Clear break between the last post and the composer: a
                // divider centered in the gap so the reply box reads as
                // its own section rather than another post.
                Rectangle {
                    y: 24
                    width: postList.width - 32
                    height: 1
                    color: Theme.outlineVariant
                }

                ReplyComposer {
                    id: replyComposer
                    y: 48
                    width: postList.width - 32
                    site: root.site
                    threadWebUrl: controller.threadWebUrl
                    onPosted: controller.reload()
                }
            }
        }
    }
}
