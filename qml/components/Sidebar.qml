import QtQuick
import QtQuick.Controls.Basic
import QtQuick.Layouts
import Channex

// M3 standard navigation drawer: sits directly on the window's
// surfaceContainer background (the content pane beside it is the
// raised `surface`), with site switcher, the active site's boards, and
// the app-level destinations pinned at the bottom.
Item {
    id: root
    width: 264

    ColumnLayout {
        anchors.fill: parent
        anchors.leftMargin: 12
        anchors.rightMargin: 12
        anchors.bottomMargin: 12
        spacing: 4

        Text {
            Layout.leftMargin: 16
            Layout.topMargin: 4
            text: "Sites"
            color: Theme.onSurfaceVariantColor
            font.pixelSize: Theme.titleSmall
            font.weight: Font.Medium
        }

        SiteSwitcher {
            Layout.fillWidth: true
            Layout.leftMargin: 12
            Layout.topMargin: 6
            Layout.bottomMargin: 10
        }

        Rectangle { Layout.fillWidth: true; Layout.leftMargin: 16; Layout.rightMargin: 16; implicitHeight: 1; color: Theme.outlineVariant }

        RowLayout {
            Layout.fillWidth: true
            Layout.leftMargin: 16
            Layout.rightMargin: 16
            Layout.topMargin: 10
            Layout.bottomMargin: 2
            Text {
                Layout.fillWidth: true
                text: "Boards"
                color: Theme.onSurfaceVariantColor
                font.pixelSize: Theme.titleSmall
                font.weight: Font.Medium
            }
            Text {
                text: boardList.count > 0 ? boardList.count : ""
                color: Theme.outline
                font.pixelSize: Theme.labelMedium
            }
        }

        BoardList { id: boardList; Layout.fillWidth: true; Layout.fillHeight: true }

        Rectangle { Layout.fillWidth: true; Layout.leftMargin: 16; Layout.rightMargin: 16; implicitHeight: 1; color: Theme.outlineVariant; Layout.topMargin: 4; Layout.bottomMargin: 4 }

        Repeater {
            model: [
                { label: "Bookmarks", view: "bookmarks", icon: "bookmark" },
                { label: "Downloads", view: "downloads", icon: "download" },
                { label: "Settings", view: "settings", icon: "gear" },
            ]
            delegate: NavItem {
                Layout.fillWidth: true
                text: modelData.label
                iconName: modelData.view === "bookmarks" && selected ? "bookmark-fill" : modelData.icon
                selected: NavigationController.view === modelData.view
                badge: modelData.view === "bookmarks" && BookmarksController.bookmarks.length > 0
                       ? String(BookmarksController.bookmarks.length)
                       : (modelData.view === "downloads" && DownloadManager.jobs.length > 0 ? String(DownloadManager.jobs.length) : "")
                onClicked: {
                    if (modelData.view === "bookmarks") NavigationController.goBookmarks()
                    else if (modelData.view === "downloads") NavigationController.goDownloads()
                    else NavigationController.goSettings()
                }
            }
        }
    }
}
