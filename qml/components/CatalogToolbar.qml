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
        Layout.fillWidth: true
        placeholderText: "Search this board…"
        text: root.controller.searchQuery
        onTextEdited: root.controller.searchQuery = text
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
                { mode: "grid", glyph: "▦" },
                { mode: "compact", glyph: "▤" },
                { mode: "list", glyph: "☰" },
            ]
            delegate: Rectangle {
                width: 32; height: 32; radius: Theme.radiusSm
                readonly property bool current: SettingsManager.catalogViewMode === modelData.mode
                color: current ? Theme.surface3 : (modeHover.hovered ? Theme.surface2 : "transparent")

                Behavior on color { ColorAnimation { duration: 120; easing.type: Easing.OutCubic } }

                Text {
                    anchors.centerIn: parent
                    text: modelData.glyph
                    color: parent.current ? Theme.accent : Theme.ink
                }
                HoverHandler { id: modeHover }
                MouseArea { anchors.fill: parent; onClicked: SettingsManager.catalogViewMode = modelData.mode }
            }
        }
    }

    AppButton {
        text: "⟳"
        flat: true
        onClicked: root.controller.reload()
    }
}
