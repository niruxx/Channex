import QtQuick
import QtQuick.Controls.Basic
import QtQuick.Layouts
import QtQuick.Dialogs
import Channex

// Mirrors src/components/onboarding/OnboardingWizard.tsx: a 5-step
// linear wizard shown until SettingsManager.hasCompletedOnboarding.
Rectangle {
    id: root
    color: Theme.canvas

    readonly property var steps: ["welcome", "appearance", "content", "downloads", "finish"]
    property int stepIndex: 0

    ColumnLayout {
        anchors.centerIn: parent
        width: Math.min(480, parent.width - 48)
        spacing: 20

        Row {
            Layout.alignment: Qt.AlignHCenter
            spacing: 6
            Repeater {
                model: root.steps.length
                delegate: Rectangle {
                    width: 8; height: 8; radius: 4
                    color: index === root.stepIndex ? Theme.accent : Theme.surface3
                }
            }
        }

        StackLayout {
            Layout.fillWidth: true
            currentIndex: root.stepIndex

            ColumnLayout {
                spacing: 10
                Text { text: "Welcome to Channex"; color: Theme.ink; font.pixelSize: 20; font.bold: true; Layout.alignment: Qt.AlignHCenter }
                Text {
                    Layout.fillWidth: true
                    horizontalAlignment: Text.AlignHCenter
                    wrapMode: Text.WordWrap
                    color: Theme.inkDim
                    text: "Browse & switch imageboards, bookmark threads, and reply & download - all from one native window."
                }
            }

            ColumnLayout {
                spacing: 10
                Text { text: "Appearance"; color: Theme.ink; font.bold: true; font.pixelSize: 16 }
                RowLayout {
                    spacing: 6
                    Repeater {
                        model: ["dark", "light", "system"]
                        delegate: AppButton { text: modelData; checkable: true; checked: SettingsManager.theme === modelData; onClicked: SettingsManager.theme = modelData }
                    }
                }
                Row {
                    spacing: 8
                    Repeater {
                        model: Theme.accentPresets
                        delegate: Rectangle {
                            width: 26; height: 26; radius: 13
                            color: modelData.value
                            border.width: SettingsManager.accentColor === modelData.value ? 3 : 0
                            border.color: Theme.ink
                            MouseArea { anchors.fill: parent; onClicked: SettingsManager.accentColor = modelData.value }
                        }
                    }
                }
                RowLayout {
                    spacing: 6
                    Repeater {
                        model: ["none", "aurora", "particles", "grid"]
                        delegate: AppButton { text: modelData; checkable: true; checked: SettingsManager.backgroundTheme === modelData; onClicked: SettingsManager.backgroundTheme = modelData }
                    }
                }
            }

            ColumnLayout {
                spacing: 10
                Text { text: "Content"; color: Theme.ink; font.bold: true; font.pixelSize: 16 }
                AppCheckBox { text: "Blur NSFW thumbnails and spoilers"; checked: SettingsManager.blurNsfw; onToggled: SettingsManager.blurNsfw = checked }
                AppCheckBox { text: "Hide NSFW sites from the switcher"; checked: SettingsManager.hideNsfwSites; onToggled: SettingsManager.hideNsfwSites = checked }
            }

            ColumnLayout {
                spacing: 10
                Text { text: "Downloads"; color: Theme.ink; font.bold: true; font.pixelSize: 16 }
                Text { color: Theme.inkDim; wrapMode: Text.WordWrap; Layout.fillWidth: true; text: "Pick a default download folder (optional - you can skip this and set it later in Settings)." }
                RowLayout {
                    Text {
                        Layout.fillWidth: true
                        color: Theme.inkDim
                        elide: Text.ElideMiddle
                        text: SettingsManager.downloadDir.length ? SettingsManager.downloadDir : "No folder chosen"
                    }
                    AppButton { text: "Choose…"; onClicked: onboardingFolderDialog.open() }
                }
            }

            ColumnLayout {
                spacing: 10
                Text { text: "You're all set"; color: Theme.ink; font.pixelSize: 20; font.bold: true; Layout.alignment: Qt.AlignHCenter }
                Text { color: Theme.inkDim; Layout.alignment: Qt.AlignHCenter; text: "Start browsing whenever you're ready." }
            }
        }

        RowLayout {
            Layout.fillWidth: true
            AppButton {
                text: "Skip setup"
                flat: true
                visible: root.stepIndex < root.steps.length - 1
                onClicked: SettingsManager.hasCompletedOnboarding = true
            }
            Item { Layout.fillWidth: true }
            AppButton {
                text: "Back"
                flat: true
                enabled: root.stepIndex > 0
                onClicked: root.stepIndex--
            }
            AppButton {
                text: root.stepIndex === root.steps.length - 1 ? "Start browsing" : "Continue"
                checked: true
                onClicked: root.stepIndex === root.steps.length - 1
                    ? SettingsManager.hasCompletedOnboarding = true
                    : root.stepIndex++
            }
        }
    }

    FolderDialog {
        id: onboardingFolderDialog
        onAccepted: SettingsManager.setDownloadDirFromUrl(selectedFolder)
    }
}
