import QtQuick
import QtQuick.Controls.Basic
import QtQuick.Layouts
import Channex

// Mirrors src/components/catalog/CatalogToolbar.tsx + ViewModeToggle.
RowLayout {
    id: root
    required property CatalogController controller
    spacing: 8

    AppTextField {
        id: searchField
        Layout.fillWidth: true
        placeholderText: "Search this board… or paste a thread link"
        text: root.controller.searchQuery
        onTextEdited: root.controller.searchQuery = text
        leftIcon: "search"

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

    AppComboBox {
        implicitWidth: 140
        model: ["Latest", "Replies", "Images", "Oldest"]
        readonly property var modes: ["bump", "replies", "images", "oldest"]
        currentIndex: modes.indexOf(root.controller.sortMode)
        onActivated: root.controller.sortMode = modes[currentIndex]
    }

    Row {
        spacing: 2
        Repeater {
            model: [
                { mode: "grid", icon: "grid" },
                { mode: "compact", icon: "rows" },
                { mode: "list", icon: "list" },
            ]
            delegate: Rectangle {
                width: 32; height: 32; radius: Theme.radiusSm
                readonly property bool current: SettingsManager.catalogViewMode === modelData.mode
                color: current ? Theme.surface3 : (modeHover.hovered ? Theme.surface2 : "transparent")

                Behavior on color { ColorAnimation { duration: 120; easing.type: Easing.OutCubic } }

                AppIcon {
                    anchors.centerIn: parent
                    name: modelData.icon
                    iconSize: 16
                    color: parent.current ? Theme.accent : Theme.ink
                    Behavior on color { ColorAnimation { duration: 120 } }
                }
                HoverHandler { id: modeHover }
                MouseArea { anchors.fill: parent; onClicked: SettingsManager.catalogViewMode = modelData.mode }
            }
        }
    }

    AppButton {
        flat: true
        iconName: "refresh"
        onClicked: root.controller.reload()
    }
}
