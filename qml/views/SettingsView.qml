import QtQuick
import QtQuick.Controls.Basic
import QtQuick.Layouts
import QtQuick.Dialogs
import Channex
import "../components"

// Settings page: a centered, max-width column of M3 settings groups
// (SettingsSection / SettingsRow). Choices use segmented buttons or
// filter chips, booleans use switches.
Flickable {
    id: root
    contentWidth: width
    contentHeight: centeredColumn.implicitHeight + 64
    clip: true
    boundsBehavior: Flickable.StopAtBounds
    ScrollBar.vertical: AppScrollBar {}

    readonly property int cardWidth: Math.min(720, width - 64)

    ColumnLayout {
        id: centeredColumn
        x: Math.round((root.width - width) / 2)
        y: 36
        width: root.cardWidth
        spacing: 28

        // ---- Header ----
        ColumnLayout {
            Layout.fillWidth: true
            Layout.leftMargin: 4
            spacing: 4
            Text { text: "Settings"; color: Theme.onSurfaceColor; font.pixelSize: Theme.headlineMedium }
            Text { text: "Appearance, content, downloads, and sites"; color: Theme.onSurfaceVariantColor; font.pixelSize: Theme.bodyLarge }
        }

        // ---- Appearance ----
        SettingsSection {
            Layout.fillWidth: true
            title: "Appearance"
            iconName: "palette"

            SettingsRow {
                divider: false
                title: "Theme"
                supportingText: "Material color scheme generated from your accent"
                trailing: SegmentedButtons {
                    model: [
                        { value: "light", label: "Light" },
                        { value: "dark", label: "Dark" },
                        { value: "system", label: "System" },
                    ]
                    current: SettingsManager.theme
                    onActivated: (v) => SettingsManager.theme = v
                }
            }

            SettingsRow {
                title: "Accent color"
                supportingText: Theme.isChanTheme ? "Classic skins use their own traditional colors" : "Seeds the whole palette"

                Flow {
                    Layout.fillWidth: true
                    spacing: 12
                    opacity: Theme.isChanTheme ? 0.45 : 1
                    Repeater {
                        model: Theme.accentPresets
                        delegate: Rectangle {
                            id: swatch
                            readonly property bool current: SettingsManager.accentColor === modelData.value
                            width: 40; height: 40; radius: 20
                            color: modelData.value
                            border.width: current ? 3 : 0
                            border.color: Theme.surfaceContainerLow

                            // Outer selection ring.
                            Rectangle {
                                anchors.fill: parent
                                anchors.margins: -3
                                radius: width / 2
                                color: "transparent"
                                border.width: 2
                                border.color: Theme.onSurfaceColor
                                visible: swatch.current
                            }
                            AppIcon {
                                anchors.centerIn: parent
                                visible: swatch.current
                                name: "check"
                                iconSize: 18
                                color: Theme.lstar(Qt.color(modelData.value)) > 60 ? "#000000" : "#ffffff"
                            }
                            StateLayer { radius: 20; layerColor: "#ffffff"; hovered: swatchHover.hovered }
                            HoverHandler { id: swatchHover }
                            MouseArea {
                                anchors.fill: parent
                                cursorShape: Qt.PointingHandCursor
                                onClicked: {
                                    SettingsManager.accentColor = modelData.value
                                    if (Theme.isChanTheme) SettingsManager.theme = "system"
                                }
                            }
                            ToolTip {
                                visible: swatchHover.hovered
                                delay: 500
                                text: modelData.name
                                padding: 8; topPadding: 4; bottomPadding: 4
                                contentItem: Text { text: modelData.name; color: Theme.inverseOnSurface; font.pixelSize: Theme.bodySmall }
                                background: Rectangle { radius: Theme.shapeExtraSmall; color: Theme.inverseSurface }
                            }
                        }
                    }
                }
            }

            // Styled after 4chan's own theme-selector footer: a set of
            // named skins, each with its signature accent as a swatch.
            // Picking one replaces the generated scheme with that
            // skin's authentic surfaces + link color (see Theme.qml).
            SettingsRow {
                title: "Classic imageboard skins"
                supportingText: "Authentic 4chan-style palettes"

                Flow {
                    Layout.fillWidth: true
                    spacing: 8
                    Repeater {
                        model: Theme.chanThemeIds
                        delegate: AppChip {
                            text: Theme.chanThemeNames[modelData]
                            swatch: Theme.chanPalettes[modelData].accent
                            checkable: true
                            checked: SettingsManager.theme === modelData
                            onClicked: SettingsManager.theme = checked ? "system" : modelData
                        }
                    }
                }
            }

            SettingsRow {
                title: "Animated background"
                trailing: SegmentedButtons {
                    model: [
                        { value: "none", label: "None" },
                        { value: "aurora", label: "Aurora" },
                        { value: "particles", label: "Particles" },
                        { value: "grid", label: "Grid" },
                    ]
                    current: SettingsManager.backgroundTheme
                    onActivated: (v) => SettingsManager.backgroundTheme = v
                }
            }

            SettingsRow {
                title: "Catalog view"
                trailing: SegmentedButtons {
                    model: [
                        { value: "grid", label: "Grid", icon: "grid" },
                        { value: "compact", label: "Compact", icon: "rows" },
                        { value: "list", label: "List", icon: "list" },
                    ]
                    current: SettingsManager.catalogViewMode
                    onActivated: (v) => SettingsManager.catalogViewMode = v
                }
            }

            SettingsRow {
                title: "Reply style"
                trailing: SegmentedButtons {
                    model: [
                        { value: "compact", label: "Compact" },
                        { value: "standard", label: "Standard" },
                        { value: "modern", label: "Modern" },
                    ]
                    current: SettingsManager.replyDisplayStyle
                    onActivated: (v) => SettingsManager.replyDisplayStyle = v
                }
            }
        }

        // ---- Content ----
        SettingsSection {
            Layout.fillWidth: true
            title: "Content"
            iconName: "shield"

            SettingsRow {
                divider: false
                title: "Blur NSFW thumbnails and spoilers"
                supportingText: "Tap a thumbnail to reveal it"
                trailing: AppSwitch { checked: SettingsManager.blurNsfw; onToggled: SettingsManager.blurNsfw = checked }
            }
            SettingsRow {
                title: "Hide NSFW sites"
                supportingText: "Keep them out of the site switcher"
                trailing: AppSwitch { checked: SettingsManager.hideNsfwSites; onToggled: SettingsManager.hideNsfwSites = checked }
            }
            SettingsRow {
                title: "Mute videos by default"
                trailing: AppSwitch { checked: SettingsManager.muteWebmsByDefault; onToggled: SettingsManager.muteWebmsByDefault = checked }
            }
            SettingsRow {
                title: "Birthday hats on October 1st"
                trailing: AppSwitch { checked: SettingsManager.birthdayHats; onToggled: SettingsManager.birthdayHats = checked }
            }
        }

        // ---- Downloads ----
        SettingsSection {
            Layout.fillWidth: true
            title: "Downloads"
            iconName: "download"

            SettingsRow {
                divider: false
                title: "Download folder"
                supportingText: SettingsManager.downloadDir.length ? SettingsManager.downloadDir : "No folder chosen"
                trailing: AppButton { variant: "tonal"; text: "Choose"; iconName: "folder"; onClicked: folderDialog.open() }
            }

            SettingsRow {
                title: "Thread folder names"
                supportingText: SettingsManager.downloadFolderNaming === "name"
                    ? "Named after the thread's subject (or its opening post)"
                    : "Named site-board-threadId, e.g. 4chan-g-105076684"
                trailing: SegmentedButtons {
                    model: [
                        { value: "id", label: "Thread ID" },
                        { value: "name", label: "Thread name" },
                    ]
                    current: SettingsManager.downloadFolderNaming
                    onActivated: (v) => SettingsManager.downloadFolderNaming = v
                }
            }
        }

        // ---- Custom sites ----
        SettingsSection {
            Layout.fillWidth: true
            title: "Custom sites"
            iconName: "globe"

            SettingsRow {
                divider: false
                visible: SettingsManager.customSites.length === 0
                title: "No custom sites"
                supportingText: "Add one with the + button in the site switcher."
            }

            Repeater {
                model: SettingsManager.customSites
                delegate: SettingsRow {
                    divider: index > 0
                    title: modelData.name
                    supportingText: modelData.origin || ""
                    trailing: AppButton {
                        variant: "icon"
                        iconName: "trash"
                        tooltip: "Remove site"
                        onClicked: SitesController.removeCustomSite(modelData.id)
                    }
                }
            }
        }

        // ---- Boards ----
        SettingsSection {
            Layout.fillWidth: true
            title: "Boards"
            iconName: "list"

            SettingsRow {
                divider: false
                title: "Sidebar boards"
                supportingText: "Hide specific boards from a site's list in the navigation drawer."

                AppComboBox {
                    id: boardsSiteBox
                    Layout.fillWidth: true
                    label: "Site"
                    model: SitesController.sites.map(function (s) { return s.name })
                }

                ListView {
                    id: boardsManageList
                    Layout.fillWidth: true
                    Layout.preferredHeight: Math.max(48, Math.min(320, contentHeight))
                    clip: true
                    spacing: 0
                    ScrollBar.vertical: AppScrollBar {}

                    readonly property var managedSite: SitesController.sites[boardsSiteBox.currentIndex] || null

                    // boardsForSite() is a plain invokable call, so this
                    // binding alone wouldn't know to recompute once its
                    // background fetch finishes - without refreshTick as
                    // an explicit dependency, this would get stuck
                    // showing just the synchronous defaultBoards
                    // fallback instead of the site's full board list.
                    property int refreshTick: 0
                    model: {
                        refreshTick
                        return managedSite ? SitesController.boardsForSite(managedSite.id) : []
                    }

                    Connections {
                        target: SitesController
                        function onBoardsForSiteChanged(siteId) {
                            if (!boardsManageList.managedSite || siteId !== boardsManageList.managedSite.id) return
                            // boardsForSite() can itself call loadBoards(),
                            // which emits this signal synchronously before
                            // returning - incrementing refreshTick directly
                            // here would mutate a dependency of the model
                            // binding while it's still being evaluated
                            // (a real "binding loop detected" warning, not
                            // just a theoretical one). Qt.callLater defers
                            // it past the current evaluation.
                            Qt.callLater(function () { boardsManageList.refreshTick++ })
                        }
                    }
                    // Reads hiddenBoards directly in each delegate too
                    // (see BoardList.qml) so checkboxes stay in sync
                    // with each other when toggled.

                    delegate: AppCheckBox {
                        width: boardsManageList.width
                        readonly property var hiddenForManagedSite: SettingsManager.hiddenBoards[boardsManageList.managedSite ? boardsManageList.managedSite.id : ""] || []
                        text: "/" + modelData.code + "/  " + (modelData.title || "")
                        checked: hiddenForManagedSite.indexOf(modelData.code) === -1
                        onToggled: SettingsManager.setBoardHidden(boardsManageList.managedSite.id, modelData.code, !checked)
                    }

                    Text {
                        anchors.centerIn: parent
                        visible: boardsManageList.count === 0
                        text: "No boards yet - open this site once to discover its boards."
                        color: Theme.onSurfaceVariantColor
                        font.pixelSize: Theme.bodyMedium
                    }
                }
            }
        }

        AppButton {
            Layout.alignment: Qt.AlignHCenter
            variant: "outlined"
            text: "Replay first-time setup"
            iconName: "refresh"
            onClicked: SettingsManager.hasCompletedOnboarding = false
        }

        ColumnLayout {
            Layout.fillWidth: true
            spacing: 2
            Text { Layout.alignment: Qt.AlignHCenter; text: "niruxxdaboi"; color: Theme.outline; font.pixelSize: Theme.labelMedium; font.weight: Font.Medium }
            Text { Layout.alignment: Qt.AlignHCenter; text: "Channex · Qt 6 · Ver. 0.1.2"; color: Theme.outline; font.pixelSize: Theme.labelSmall }
        }

        Item { Layout.preferredHeight: 8 }
    }

    FolderDialog {
        id: folderDialog
        onAccepted: SettingsManager.setDownloadDirFromUrl(selectedFolder)
    }
}
