import QtQuick
import QtQuick.Controls.Basic
import QtQuick.Layouts
import Channex

// M3 search bar + a sort chip (with menu) + view-mode segmented
// buttons + refresh.
RowLayout {
    id: root
    required property CatalogController controller
    spacing: 12

    AppTextField {
        id: searchField
        Layout.fillWidth: true
        Layout.maximumWidth: 720
        variant: "search"
        placeholderText: "Search this board, or paste a thread link"
        text: root.controller.searchQuery
        onTextEdited: root.controller.searchQuery = text

        // Paste a thread URL (4chan "/board/thread/id" or vichan/
        // LynxChan "/board/res/id.html") and hit Enter to jump straight
        // to it, instead of it just filtering the current board's
        // catalog. Only acts when the text resolves against a site
        // that's already registered (preset or custom) - anything else
        // is left alone as a normal search term.
        onAccepted: {
            const resolved = SitesController.resolveThreadUrl(text)
            if (resolved && resolved.threadId) {
                SitesController.currentSiteId = resolved.siteId
                NavigationController.goThread(resolved.siteId, resolved.boardCode, resolved.threadId)
                text = ""
                root.controller.searchQuery = ""
            }
        }
    }

    Item { Layout.fillWidth: true }

    AppChip {
        id: sortChip
        readonly property var modes: ["bump", "replies", "images", "oldest"]
        readonly property var labels: ["Latest bump", "Most replies", "Most images", "Oldest"]
        text: labels[Math.max(0, modes.indexOf(root.controller.sortMode))]
        iconName: "sort"
        onClicked: sortMenu.popup(sortChip, 0, sortChip.height + 4)

        AppMenu {
            id: sortMenu
            Repeater {
                model: sortChip.modes.length
                delegate: AppMenuItem {
                    text: sortChip.labels[index]
                    iconName: root.controller.sortMode === sortChip.modes[index] ? "check" : ""
                    onTriggered: root.controller.sortMode = sortChip.modes[index]
                }
            }
        }
    }

    SegmentedButtons {
        compact: false
        model: [
            { value: "grid", icon: "grid" },
            { value: "compact", icon: "rows" },
            { value: "list", icon: "list" },
        ]
        current: SettingsManager.catalogViewMode
        onActivated: (v) => SettingsManager.catalogViewMode = v
    }

    AppButton {
        variant: "icon"
        iconName: "refresh"
        tooltip: "Refresh"
        onClicked: root.controller.reload()
    }
}
