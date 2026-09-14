import QtQuick
import QtQuick.Controls.Basic
import QtQuick.Layouts
import QtQuick.Dialogs
import Channex
import "../components"

// Mirrors src/components/settings/SettingsPanel.tsx (+ ThemePicker,
// AccentPicker, BackgroundPicker). Laid out as a centered, max-width
// column of section "cards" rather than a flat left-aligned list, so it
// reads as a real settings page instead of a stack of controls jammed
// in the corner.
Flickable {
    id: root
    contentWidth: width
    contentHeight: centeredColumn.implicitHeight + 64
    clip: true
    boundsBehavior: Flickable.StopAtBounds
    ScrollBar.vertical: ScrollBar {}

    readonly property int cardWidth: Math.min(640, width - 48)

    Column {
        id: centeredColumn
        anchors.horizontalCenter: parent.horizontalCenter
        y: 32
        width: root.cardWidth
        spacing: 20

        // ---- Header ----
        RowLayout {
            width: parent.width
            spacing: 12

            Image {
                Layout.preferredWidth: 40
                Layout.preferredHeight: 40
                source: "qrc:/qt/qml/Channex/resources/icons/app.svg"
                fillMode: Image.PreserveAspectFit
                smooth: true
            }
            ColumnLayout {
                spacing: 2
                Text { text: "Settings"; color: Theme.ink; font.bold: true; font.pixelSize: 22 }
                Text { text: "Appearance, content, downloads, and sites"; color: Theme.inkFaint; font.pixelSize: 13 }
            }
            Item { Layout.fillWidth: true }
        }

        // ---- Appearance ----
        Rectangle {
            width: parent.width
            height: appearanceCol.implicitHeight + 32
            radius: Theme.radiusLg
            color: Theme.surface
            border.color: Theme.border

            CardShadow { anchors.fill: parent; radius: Theme.radiusLg }

            ColumnLayout {
                id: appearanceCol
                x: 20; y: 16
                width: parent.width - 40
                spacing: 14

                Text { text: "Appearance"; color: Theme.ink; font.bold: true; font.pixelSize: 15 }

                ColumnLayout {
                    spacing: 6
                    Text { text: "Theme"; color: Theme.inkDim; font.pixelSize: 12 }
                    RowLayout {
                        spacing: 6
                        Repeater {
                            model: ["dark", "light", "system"]
                            delegate: AppButton {
                                text: modelData.charAt(0).toUpperCase() + modelData.slice(1)
                                checkable: true
                                checked: SettingsManager.theme === modelData
                                onClicked: SettingsManager.theme = modelData
                            }
                        }
                    }
                }

                ColumnLayout {
                    spacing: 6
                    Text { text: "Accent color"; color: Theme.inkDim; font.pixelSize: 12 }
                    Row {
                        spacing: 10
                        Repeater {
                            model: Theme.accentPresets
                            delegate: Rectangle {
                                width: 28; height: 28; radius: 14
                                color: modelData.value
                                border.width: SettingsManager.accentColor === modelData.value ? 3 : 0
                                border.color: Theme.ink
                                scale: swatchHover.hovered ? 1.12 : 1.0
                                Behavior on scale { NumberAnimation { duration: 100; easing.type: Easing.OutCubic } }
                                HoverHandler { id: swatchHover }
                                MouseArea { anchors.fill: parent; cursorShape: Qt.PointingHandCursor; onClicked: SettingsManager.accentColor = modelData.value }
                            }
                        }
                    }
                }

                ColumnLayout {
                    spacing: 6
                    Text { text: "Animated background"; color: Theme.inkDim; font.pixelSize: 12 }
                    RowLayout {
                        spacing: 6
                        Repeater {
                            model: ["none", "aurora", "particles", "grid"]
                            delegate: AppButton {
                                text: modelData.charAt(0).toUpperCase() + modelData.slice(1)
                                checkable: true
                                checked: SettingsManager.backgroundTheme === modelData
                                onClicked: SettingsManager.backgroundTheme = modelData
                            }
                        }
                    }
                }

                ColumnLayout {
                    spacing: 6
                    Text { text: "Catalog view"; color: Theme.inkDim; font.pixelSize: 12 }
                    RowLayout {
                        spacing: 6
                        Repeater {
                            model: [
                                { mode: "grid", icon: "grid", label: "Grid" },
                                { mode: "compact", icon: "rows", label: "Compact" },
                                { mode: "list", icon: "list", label: "List" },
                            ]
                            delegate: AppButton {
                                text: modelData.label
                                iconName: modelData.icon
                                checkable: true
                                checked: SettingsManager.catalogViewMode === modelData.mode
                                onClicked: SettingsManager.catalogViewMode = modelData.mode
                            }
                        }
                    }
                }

                ColumnLayout {
                    spacing: 6
                    Text { text: "Reply style"; color: Theme.inkDim; font.pixelSize: 12 }
                    RowLayout {
                        spacing: 6
                        Repeater {
                            model: [
                                { mode: "compact", label: "Compact" },
                                { mode: "standard", label: "Standard" },
                                { mode: "modern", label: "Modern" },
                            ]
                            delegate: AppButton {
                                text: modelData.label
                                checkable: true
                                checked: SettingsManager.replyDisplayStyle === modelData.mode
                                onClicked: SettingsManager.replyDisplayStyle = modelData.mode
                            }
                        }
                    }
                }
            }
        }

        // ---- Content ----
        Rectangle {
            width: parent.width
            height: contentCol.implicitHeight + 32
            radius: Theme.radiusLg
            color: Theme.surface
            border.color: Theme.border

            CardShadow { anchors.fill: parent; radius: Theme.radiusLg }

            ColumnLayout {
                id: contentCol
                x: 20; y: 16
                width: parent.width - 40
                spacing: 10

                Text { text: "Content"; color: Theme.ink; font.bold: true; font.pixelSize: 15 }
                AppCheckBox { text: "Blur NSFW thumbnails and spoilers until tapped"; checked: SettingsManager.blurNsfw; onToggled: SettingsManager.blurNsfw = checked }
                AppCheckBox { text: "Hide NSFW sites from the switcher"; checked: SettingsManager.hideNsfwSites; onToggled: SettingsManager.hideNsfwSites = checked }
                AppCheckBox { text: "Mute WebMs/videos by default"; checked: SettingsManager.muteWebmsByDefault; onToggled: SettingsManager.muteWebmsByDefault = checked }
                AppCheckBox { text: "Show birthday hats on October 1st"; checked: SettingsManager.birthdayHats; onToggled: SettingsManager.birthdayHats = checked }
            }
        }

        // ---- Downloads ----
        Rectangle {
            width: parent.width
            height: downloadsCol.implicitHeight + 32
            radius: Theme.radiusLg
            color: Theme.surface
            border.color: Theme.border

            CardShadow { anchors.fill: parent; radius: Theme.radiusLg }

            ColumnLayout {
                id: downloadsCol
                x: 20; y: 16
                width: parent.width - 40
                spacing: 10

                Text { text: "Downloads"; color: Theme.ink; font.bold: true; font.pixelSize: 15 }
                RowLayout {
                    Layout.fillWidth: true
                    spacing: 10
                    Rectangle {
                        Layout.fillWidth: true
                        implicitHeight: 36
                        radius: Theme.radiusSm
                        color: Theme.surface2
                        border.color: Theme.border
                        Text {
                            anchors.fill: parent
                            anchors.leftMargin: 12
                            anchors.rightMargin: 12
                            verticalAlignment: Text.AlignVCenter
                            text: SettingsManager.downloadDir.length ? SettingsManager.downloadDir : "No folder chosen"
                            color: Theme.inkDim
                            elide: Text.ElideMiddle
                        }
                    }
                    AppButton { text: "Choose…"; iconName: "folder"; onClicked: folderDialog.open() }
                }
            }
        }

        // ---- Custom sites ----
        Rectangle {
            width: parent.width
            height: sitesCol.implicitHeight + 32
            radius: Theme.radiusLg
            color: Theme.surface
            border.color: Theme.border

            CardShadow { anchors.fill: parent; radius: Theme.radiusLg }

            ColumnLayout {
                id: sitesCol
                x: 20; y: 16
                width: parent.width - 40
                spacing: 10

                Text { text: "Custom sites"; color: Theme.ink; font.bold: true; font.pixelSize: 15 }
                Repeater {
                    model: SettingsManager.customSites
                    delegate: RowLayout {
                        Layout.fillWidth: true
                        Text { text: modelData.name; color: Theme.ink; Layout.fillWidth: true }
                        AppButton { text: "Remove"; flat: true; onClicked: SitesController.removeCustomSite(modelData.id) }
                    }
                }
                Text {
                    visible: SettingsManager.customSites.length === 0
                    text: "Add one from the \"+\" button next to the site switcher."
                    color: Theme.inkFaint
                    font.pixelSize: 12
                }
            }
        }

        // ---- Boards ----
        Rectangle {
            width: parent.width
            height: boardsCol.implicitHeight + 32
            radius: Theme.radiusLg
            color: Theme.surface
            border.color: Theme.border

            CardShadow { anchors.fill: parent; radius: Theme.radiusLg }

            ColumnLayout {
                id: boardsCol
                x: 20; y: 16
                width: parent.width - 40
                spacing: 10

                Text { text: "Boards"; color: Theme.ink; font.bold: true; font.pixelSize: 15 }
                Text {
                    Layout.fillWidth: true
                    wrapMode: Text.WordWrap
                    color: Theme.inkDim
                    font.pixelSize: 12
                    text: "Hide specific boards from a site's list in the sidebar."
                }

                AppComboBox {
                    id: boardsSiteBox
                    Layout.fillWidth: true
                    model: SitesController.sites.map(function (s) { return s.name })
                }

                ListView {
                    id: boardsManageList
                    Layout.fillWidth: true
                    Layout.preferredHeight: Math.min(280, contentHeight)
                    clip: true
                    spacing: 2

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

                    delegate: RowLayout {
                        width: boardsManageList.width
                        readonly property var hiddenForManagedSite: SettingsManager.hiddenBoards[boardsManageList.managedSite ? boardsManageList.managedSite.id : ""] || []
                        readonly property bool shown: hiddenForManagedSite.indexOf(modelData.code) === -1

                        AppCheckBox {
                            Layout.fillWidth: true
                            text: "/" + modelData.code + "/  " + (modelData.title || "")
                            checked: shown
                            onToggled: SettingsManager.setBoardHidden(boardsManageList.managedSite.id, modelData.code, !checked)
                        }
                    }

                    Text {
                        anchors.centerIn: parent
                        visible: boardsManageList.count === 0
                        text: "No boards yet - open this site once to discover its boards."
                        color: Theme.inkFaint
                        font.pixelSize: 12
                    }
                }
            }
        }

        AppButton {
            anchors.horizontalCenter: parent.horizontalCenter
            text: "Replay first-time setup"
            flat: true
            onClicked: SettingsManager.hasCompletedOnboarding = false
        }

        ColumnLayout {
            Layout.fillWidth: true
            Layout.topMargin: 8
            spacing: 2
            Text { Layout.alignment: Qt.AlignHCenter; text: "- niruxxdaboi -"; color: Theme.inkFaint; font.pixelSize: 12 }
            Text { Layout.alignment: Qt.AlignHCenter; text: "QT6 - Ver. 1.0.0"; color: Theme.inkFaint; font.pixelSize: 11 }
        }

        Item { width: 1; height: 8 }
    }

    FolderDialog {
        id: folderDialog
        onAccepted: SettingsManager.setDownloadDirFromUrl(selectedFolder)
    }
}
