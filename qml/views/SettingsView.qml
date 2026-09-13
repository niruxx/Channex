import QtQuick
import QtQuick.Controls.Basic
import QtQuick.Layouts
import QtQuick.Dialogs
import Channex
import "../components"

// Mirrors src/components/settings/SettingsPanel.tsx (+ ThemePicker,
// AccentPicker, BackgroundPicker).
Flickable {
    id: root
    contentWidth: width
    contentHeight: column.implicitHeight + 24
    clip: true

    ColumnLayout {
        id: column
        width: root.width - 24
        x: 12
        y: 12
        spacing: 20

        // ---- Appearance ----
        ColumnLayout {
            Layout.fillWidth: true
            spacing: 8
            Text { text: "Appearance"; color: Theme.ink; font.bold: true; font.pixelSize: 14 }

            RowLayout {
                spacing: 6
                Repeater {
                    model: ["dark", "light", "system"]
                    delegate: AppButton {
                        text: modelData
                        checkable: true
                        checked: SettingsManager.theme === modelData
                        onClicked: SettingsManager.theme = modelData
                    }
                }
            }

            Text { text: "Accent color"; color: Theme.inkDim; font.pixelSize: 12 }
            Row {
                spacing: 8
                Repeater {
                    model: Theme.accentPresets
                    delegate: Rectangle {
                        width: 28; height: 28; radius: 14
                        color: modelData.value
                        border.width: SettingsManager.accentColor === modelData.value ? 3 : 0
                        border.color: Theme.ink
                        MouseArea { anchors.fill: parent; onClicked: SettingsManager.accentColor = modelData.value }
                    }
                }
            }

            Text { text: "Animated background"; color: Theme.inkDim; font.pixelSize: 12 }
            RowLayout {
                spacing: 6
                Repeater {
                    model: ["none", "aurora", "particles", "grid"]
                    delegate: AppButton {
                        text: modelData
                        checkable: true
                        checked: SettingsManager.backgroundTheme === modelData
                        onClicked: SettingsManager.backgroundTheme = modelData
                    }
                }
            }

            Text { text: "Catalog view"; color: Theme.inkDim; font.pixelSize: 12 }
            RowLayout {
                spacing: 6
                Repeater {
                    model: ["grid", "compact", "list"]
                    delegate: AppButton {
                        text: modelData
                        checkable: true
                        checked: SettingsManager.catalogViewMode === modelData
                        onClicked: SettingsManager.catalogViewMode = modelData
                    }
                }
            }
        }

        Rectangle { Layout.fillWidth: true; height: 1; color: Theme.borderSoft }

        // ---- Content ----
        ColumnLayout {
            Layout.fillWidth: true
            spacing: 8
            Text { text: "Content"; color: Theme.ink; font.bold: true; font.pixelSize: 14 }
            AppCheckBox { text: "Blur NSFW thumbnails and spoilers until tapped"; checked: SettingsManager.blurNsfw; onToggled: SettingsManager.blurNsfw = checked }
            AppCheckBox { text: "Hide NSFW sites from the switcher"; checked: SettingsManager.hideNsfwSites; onToggled: SettingsManager.hideNsfwSites = checked }
            AppCheckBox { text: "Mute WebMs/videos by default"; checked: SettingsManager.muteWebmsByDefault; onToggled: SettingsManager.muteWebmsByDefault = checked }
            AppCheckBox { text: "Show birthday hats on October 1st"; checked: SettingsManager.birthdayHats; onToggled: SettingsManager.birthdayHats = checked }
        }

        Rectangle { Layout.fillWidth: true; height: 1; color: Theme.borderSoft }

        // ---- Downloads ----
        ColumnLayout {
            Layout.fillWidth: true
            spacing: 8
            Text { text: "Downloads"; color: Theme.ink; font.bold: true; font.pixelSize: 14 }
            RowLayout {
                Text {
                    text: SettingsManager.downloadDir.length ? SettingsManager.downloadDir : "No folder chosen"
                    color: Theme.inkDim
                    Layout.fillWidth: true
                    elide: Text.ElideMiddle
                }
                AppButton { text: "Choose…"; onClicked: folderDialog.open() }
            }
        }

        Rectangle { Layout.fillWidth: true; height: 1; color: Theme.borderSoft }

        // ---- Custom sites ----
        ColumnLayout {
            Layout.fillWidth: true
            spacing: 8
            Text { text: "Custom sites"; color: Theme.ink; font.bold: true; font.pixelSize: 14 }
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

        Rectangle { Layout.fillWidth: true; height: 1; color: Theme.borderSoft }

        AppButton {
            text: "Replay first-time setup"
            onClicked: SettingsManager.hasCompletedOnboarding = false
        }
    }

    FolderDialog {
        id: folderDialog
        onAccepted: SettingsManager.setDownloadDirFromUrl(selectedFolder)
    }
}
