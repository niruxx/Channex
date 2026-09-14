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

    radius: Theme.radiusMd
    color: Theme.surface2
    border.color: Theme.border
    implicitHeight: layout.implicitHeight + 24

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
        anchors.margins: 12
        spacing: 8

        Text { text: "Reply"; color: Theme.ink; font.bold: true; font.pixelSize: 13 }

        ColumnLayout {
            opacity: root.isLynxchan ? 1 : 0
            visible: opacity > 0
            Behavior on opacity { NumberAnimation { duration: 160; easing.type: Easing.OutCubic } }
            Layout.fillWidth: true
            spacing: 8

            AppTextField { id: nameField; Layout.fillWidth: true; placeholderText: "Name (optional)" }
            AppTextArea {
                id: messageField
                Layout.fillWidth: true
                Layout.preferredHeight: 90
                placeholderText: "Write a reply…"
            }
            RowLayout {
                Image {
                    source: root.captchaImage
                    width: 140; height: 40
                    fillMode: Image.PreserveAspectFit
                }
                AppButton { text: "⟳"; flat: true; onClicked: root.refreshCaptcha() }
                AppTextField { id: captchaField; Layout.fillWidth: true; placeholderText: "Captcha answer" }
            }
            RowLayout {
                Item { Layout.fillWidth: true }
                Text { text: root.statusMessage; color: Theme.inkFaint; font.pixelSize: 11 }
                AppButton {
                    text: root.busy ? "Posting…" : "Post reply"
                    checked: true
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
            spacing: 8

            Text {
                Layout.fillWidth: true
                wrapMode: Text.WordWrap
                color: Theme.inkDim
                font.pixelSize: 12
                text: "This site requires solving a verification challenge on its own page. Write your reply below, then Continue on site - it'll be copied to your clipboard to paste in, and the thread here won't auto-refresh, so hit ⟳ above when you're done."
            }
            AppTextArea {
                id: extMessageField
                Layout.fillWidth: true
                Layout.preferredHeight: 70
                placeholderText: "Write a reply to copy…"
            }
            RowLayout {
                AppButton {
                    text: "Sign in with 4chan Pass"
                    visible: root.site.id === "4chan"
                    flat: true
                    onClicked: ExternalReply.openAuthPage("https://sys.4chan.org/auth")
                }
                Item { Layout.fillWidth: true }
                AppButton {
                    text: "Continue on site"
                    checked: true
                    onClicked: ExternalReply.openForReply(root.threadWebUrl, extMessageField.text)
                }
            }
        }
    }
}
