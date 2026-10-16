import QtQuick
import QtQuick.Controls.Basic
import QtQuick.Layouts
import Channex
import "../components"

// Mirrors src/components/thread/ReplyComposer.tsx: branches on
// site.postEngine. "lynxchan" sites get an in-app captcha + form via
// LynxchanPoster; "external" sites (4chan, Lainchan) get a browser
// handoff - see ExternalReply.h for the documented parity gap
// (no auto-refresh-on-popup-close, since that needs an embedded
// webview this placeholder doesn't include yet).
Rectangle {
    id: root
    property var site: ({})
    property string threadWebUrl: ""
    signal posted()

    // M3 outlined card holding filled text fields and a filled
    // primary action.
    radius: Theme.shapeLarge
    color: Theme.surface
    border.width: 1
    border.color: Theme.outlineVariant
    implicitHeight: layout.implicitHeight + 40

    readonly property bool isLynxchan: site.postEngine === "lynxchan"

    property string captchaImage: ""
    property string statusMessage: ""
    property bool busy: false

    Connections {
        target: LynxchanPoster
        function onCaptchaReady(url) { root.captchaImage = url }
        function onCaptchaFailed(err) { root.statusMessage = "Captcha error: " + err }
        function onPostSucceeded(id) {
            root.busy = false
            root.statusMessage = "Posted."
            messageField.text = ""
            captchaField.text = ""
            root.posted()
        }
        function onPostFailed(err) {
            root.busy = false
            root.statusMessage = "Failed: " + err
            if (root.isLynxchan) LynxchanPoster.fetchCaptcha(root.site, NavigationController.boardCode)
        }
    }

    function refreshCaptcha() {
        if (root.isLynxchan) LynxchanPoster.fetchCaptcha(root.site, NavigationController.boardCode)
    }

    onIsLynxchanChanged: if (isLynxchan) refreshCaptcha()
    Component.onCompleted: if (isLynxchan) refreshCaptcha()

    ColumnLayout {
        id: layout
        anchors.fill: parent
        anchors.margins: 20
        spacing: 12

        RowLayout {
            spacing: 10
            Rectangle {
                implicitWidth: 32; implicitHeight: 32; radius: 16
                color: Theme.secondaryContainer
                AppIcon { anchors.centerIn: parent; name: "chat"; iconSize: 16; color: Theme.onSecondaryContainerColor }
            }
            Text { text: "Reply to thread"; color: Theme.onSurfaceColor; font.pixelSize: Theme.titleMedium; font.weight: Font.Medium }
        }

        ColumnLayout {
            opacity: root.isLynxchan ? 1 : 0
            visible: opacity > 0
            Behavior on opacity { NumberAnimation { duration: 160; easing.type: Easing.OutCubic } }
            Layout.fillWidth: true
            spacing: 12

            AppTextField { id: nameField; Layout.fillWidth: true; label: "Name"; placeholderText: "Anonymous" }
            AppTextArea {
                id: messageField
                Layout.fillWidth: true
                Layout.preferredHeight: 120
                label: "Comment"
            }
            RowLayout {
                spacing: 12
                Rectangle {
                    Layout.preferredWidth: 168
                    Layout.preferredHeight: 56
                    radius: Theme.shapeSmall
                    color: Theme.surfaceContainerHighest
                    Image {
                        anchors.fill: parent
                        anchors.margins: 4
                        source: root.captchaImage
                        fillMode: Image.PreserveAspectFit
                    }
                }
                AppButton { variant: "icon"; iconName: "refresh"; tooltip: "New captcha"; onClicked: root.refreshCaptcha() }
                AppTextField { id: captchaField; Layout.fillWidth: true; label: "Captcha answer" }
            }
            RowLayout {
                spacing: 12
                Text {
                    Layout.fillWidth: true
                    text: root.statusMessage
                    color: root.statusMessage.indexOf("Failed") === 0 || root.statusMessage.indexOf("Captcha error") === 0 ? Theme.error : Theme.onSurfaceVariantColor
                    font.pixelSize: Theme.bodySmall
                    elide: Text.ElideRight
                }
                AppButton {
                    text: root.busy ? "Posting…" : "Post reply"
                    variant: "filled"
                    iconName: "send"
                    enabled: !root.busy && messageField.text.length > 0
                    onClicked: {
                        root.busy = true
                        root.statusMessage = ""
                        LynxchanPoster.postReply(root.site, NavigationController.boardCode, NavigationController.threadId,
                            messageField.text, nameField.text, "", captchaField.text, false)
                    }
                }
            }
        }

        ColumnLayout {
            opacity: root.isLynxchan ? 0 : 1
            visible: opacity > 0
            Behavior on opacity { NumberAnimation { duration: 160; easing.type: Easing.OutCubic } }
            Layout.fillWidth: true
            spacing: 12

            Text {
                Layout.fillWidth: true
                wrapMode: Text.WordWrap
                color: Theme.onSurfaceVariantColor
                font.pixelSize: Theme.bodyMedium
                text: "This site requires solving a verification challenge on its own page. Write your reply below, then Continue on site - it'll be copied to your clipboard to paste in, and the thread here won't auto-refresh, so hit refresh above when you're done."
            }
            AppTextArea {
                id: extMessageField
                Layout.fillWidth: true
                Layout.preferredHeight: 110
                label: "Comment (copied to clipboard)"
            }
            RowLayout {
                spacing: 8
                AppButton {
                    text: "Sign in with 4chan Pass"
                    visible: root.site.id === "4chan"
                    variant: "text"
                    onClicked: ExternalReply.openAuthPage("https://sys.4chan.org/auth")
                }
                Item { Layout.fillWidth: true }
                AppButton {
                    text: "Continue on site"
                    variant: "filled"
                    iconName: "open-external"
                    onClicked: ExternalReply.openForReply(root.threadWebUrl, extMessageField.text)
                }
            }
        }
    }
}
