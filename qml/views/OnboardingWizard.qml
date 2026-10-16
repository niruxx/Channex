import QtQuick
import QtQuick.Controls.Basic
import QtQuick.Layouts
import QtQuick.Dialogs
import Channex
import "../components"

// A 5-step linear wizard shown until SettingsManager.hasCompletedOnboarding:
// one extra-large-radius M3 card centered on the window, a tonal hero
// icon per step, an expanding-pill step indicator, and text / filled
// actions along the bottom.
Rectangle {
    id: root
    color: Theme.surfaceContainer

    readonly property var steps: [
        { id: "welcome", icon: "grid", title: "Welcome to Channex", body: "Browse and switch imageboards, bookmark threads, reply, and download - all from one native window." },
        { id: "appearance", icon: "palette", title: "Make it yours", body: "Pick a theme and an accent color. The whole Material palette is generated from it." },
        { id: "content", icon: "shield", title: "Content", body: "Choose how sensitive content is handled. You can change this any time." },
        { id: "downloads", icon: "download", title: "Downloads", body: "Pick a default download folder - optional, you can set it later in Settings." },
        { id: "finish", icon: "check", title: "You're all set", body: "Start browsing whenever you're ready." },
    ]
    property int stepIndex: 0
    readonly property var step: steps[stepIndex]

    Rectangle {
        id: card
        anchors.centerIn: parent
        width: Math.min(560, parent.width - 48)
        height: cardColumn.implicitHeight + 64
        radius: Theme.shapeExtraLarge
        color: Theme.surface
        Behavior on height { NumberAnimation { duration: Theme.durMedium; easing.type: Easing.BezierSpline; easing.bezierCurve: Theme.easeEmphasizedDecel } }

        CardShadow { radius: card.radius; level: 1 }

        ColumnLayout {
            id: cardColumn
            x: 32; y: 32
            width: parent.width - 64
            spacing: 24

            Row {
                Layout.alignment: Qt.AlignHCenter
                spacing: 6
                Repeater {
                    model: root.steps.length
                    delegate: Rectangle {
                        width: index === root.stepIndex ? 28 : 8
                        height: 8
                        radius: 4
                        color: index <= root.stepIndex ? Theme.primary : Theme.surfaceContainerHighest
                        Behavior on width { NumberAnimation { duration: Theme.durMedium; easing.type: Easing.BezierSpline; easing.bezierCurve: Theme.easeEmphasizedDecel } }
                        Behavior on color { ColorAnimation { duration: Theme.durMedium } }
                    }
                }
            }

            Rectangle {
                Layout.alignment: Qt.AlignHCenter
                implicitWidth: 80; implicitHeight: 80
                radius: root.stepIndex % 2 === 0 ? 40 : Theme.shapeExtraLarge
                color: Theme.primaryContainer
                Behavior on radius { NumberAnimation { duration: Theme.durLong; easing.type: Easing.BezierSpline; easing.bezierCurve: Theme.easeEmphasizedDecel } }
                AppIcon { anchors.centerIn: parent; name: root.step.icon; iconSize: 36; color: Theme.onPrimaryContainerColor }
            }

            ColumnLayout {
                Layout.fillWidth: true
                spacing: 8
                Text {
                    Layout.fillWidth: true
                    horizontalAlignment: Text.AlignHCenter
                    text: root.step.title
                    color: Theme.onSurfaceColor
                    font.pixelSize: Theme.headlineSmall
                }
                Text {
                    Layout.fillWidth: true
                    horizontalAlignment: Text.AlignHCenter
                    wrapMode: Text.WordWrap
                    text: root.step.body
                    color: Theme.onSurfaceVariantColor
                    font.pixelSize: Theme.bodyLarge
                }
            }

            StackLayout {
                Layout.fillWidth: true
                currentIndex: root.stepIndex
                Layout.preferredHeight: children[currentIndex] ? children[currentIndex].implicitHeight : 0

                Item { implicitHeight: 0 }

                ColumnLayout {
                    spacing: 16
                    SegmentedButtons {
                        Layout.alignment: Qt.AlignHCenter
                        model: [
                            { value: "light", label: "Light" },
                            { value: "dark", label: "Dark" },
                            { value: "system", label: "System" },
                        ]
                        current: SettingsManager.theme
                        onActivated: (v) => SettingsManager.theme = v
                    }
                    Row {
                        Layout.alignment: Qt.AlignHCenter
                        spacing: 10
                        Repeater {
                            model: Theme.accentPresets
                            delegate: Rectangle {
                                readonly property bool current: SettingsManager.accentColor === modelData.value
                                width: 36; height: 36; radius: 18
                                color: modelData.value
                                Rectangle {
                                    anchors.fill: parent
                                    anchors.margins: -4
                                    radius: width / 2
                                    color: "transparent"
                                    border.width: 2
                                    border.color: Theme.onSurfaceColor
                                    visible: parent.current
                                }
                                AppIcon {
                                    anchors.centerIn: parent
                                    visible: parent.current
                                    name: "check"
                                    iconSize: 16
                                    color: Theme.lstar(Qt.color(modelData.value)) > 60 ? "#000000" : "#ffffff"
                                }
                                MouseArea { anchors.fill: parent; cursorShape: Qt.PointingHandCursor; onClicked: SettingsManager.accentColor = modelData.value }
                            }
                        }
                    }
                    SegmentedButtons {
                        Layout.alignment: Qt.AlignHCenter
                        compact: true
                        model: [
                            { value: "none", label: "No background" },
                            { value: "aurora", label: "Aurora" },
                            { value: "particles", label: "Particles" },
                            { value: "grid", label: "Grid" },
                        ]
                        current: SettingsManager.backgroundTheme
                        onActivated: (v) => SettingsManager.backgroundTheme = v
                    }
                }

                ColumnLayout {
                    spacing: 4
                    AppSwitch { Layout.fillWidth: true; text: "Blur NSFW thumbnails and spoilers"; checked: SettingsManager.blurNsfw; onToggled: SettingsManager.blurNsfw = checked }
                    AppSwitch { Layout.fillWidth: true; Layout.topMargin: 12; text: "Hide NSFW sites from the switcher"; checked: SettingsManager.hideNsfwSites; onToggled: SettingsManager.hideNsfwSites = checked }
                }

                RowLayout {
                    spacing: 12
                    Rectangle {
                        Layout.fillWidth: true
                        implicitHeight: 48
                        radius: Theme.shapeSmall
                        color: Theme.surfaceContainerHighest
                        RowLayout {
                            anchors.fill: parent
                            anchors.leftMargin: 14
                            anchors.rightMargin: 14
                            spacing: 10
                            AppIcon { name: "folder"; iconSize: 20; color: Theme.onSurfaceVariantColor }
                            Text {
                                Layout.fillWidth: true
                                color: SettingsManager.downloadDir.length ? Theme.onSurfaceColor : Theme.onSurfaceVariantColor
                                font.pixelSize: Theme.bodyMedium
                                elide: Text.ElideMiddle
                                text: SettingsManager.downloadDir.length ? SettingsManager.downloadDir : "No folder chosen"
                            }
                        }
                    }
                    AppButton { variant: "tonal"; text: "Choose"; onClicked: onboardingFolderDialog.open() }
                }

                Item { implicitHeight: 0 }
            }

            RowLayout {
                Layout.fillWidth: true
                spacing: 8
                AppButton {
                    variant: "text"
                    text: "Skip setup"
                    visible: root.stepIndex < root.steps.length - 1
                    onClicked: SettingsManager.hasCompletedOnboarding = true
                }
                Item { Layout.fillWidth: true }
                AppButton {
                    variant: "outlined"
                    text: "Back"
                    visible: root.stepIndex > 0
                    onClicked: root.stepIndex--
                }
                AppButton {
                    variant: "filled"
                    text: root.stepIndex === root.steps.length - 1 ? "Start browsing" : "Continue"
                    iconName: root.stepIndex === root.steps.length - 1 ? "check" : ""
                    onClicked: root.stepIndex === root.steps.length - 1
                        ? SettingsManager.hasCompletedOnboarding = true
                        : root.stepIndex++
                }
            }
        }
    }

    FolderDialog {
        id: onboardingFolderDialog
        onAccepted: SettingsManager.setDownloadDirFromUrl(selectedFolder)
    }
}
