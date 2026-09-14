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
                text: root.bookmarked ? "Bookmarked" : "Bookmark"
                flat: true
                checked: root.bookmarked
                onClicked: BookmarksController.toggle({
                    siteId: NavigationController.siteId, boardCode: NavigationController.boardCode,
                    threadId: NavigationController.threadId, subject: controller.subject,
                    replyCount: controller.postCount - 1,
                })
            }

            AppButton {
                text: "Download (" + controller.allFiles.length + ")"
                iconName: "download"
                enabled: controller.allFiles.length > 0
                flat: true
                onClicked: {
                    var destBase = SettingsManager.downloadDir.length ? SettingsManager.downloadDir : "."
                    var jobLabel = NavigationController.siteId + "-" + NavigationController.boardCode + "-" + NavigationController.threadId
                    downloadDialog.openWithFiles(controller.allFiles, jobLabel, destBase + "/" + jobLabel)
                }
            }

            AppButton { text: "Open in browser"; flat: true; onClicked: ExternalReply.openUrl(controller.threadWebUrl) }
            AppButton { iconName: "refresh"; flat: true; onClicked: controller.reload() }
        }

        DownloadSelectionDialog { id: downloadDialog }

        Rectangle { Layout.fillWidth: true; height: 1; color: Theme.borderSoft }

        ListView {
            id: postList
            Layout.fillWidth: true
            Layout.fillHeight: true
            clip: true
            model: controller
            spacing: 10
            leftMargin: 12; rightMargin: 12; topMargin: 12; bottomMargin: 12

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
                    onFileOpenRequested: (url) => {
                        var all = controller.allFiles
                        var idx = 0
                        for (var i = 0; i < all.length; i++)
                            if (all[i].url === url) { idx = i; break }
                        LightboxController.open(all, idx)
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
