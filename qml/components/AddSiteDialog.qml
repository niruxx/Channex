import QtQuick
import QtQuick.Controls.Basic
import QtQuick.Layouts
import Channex

// Mirrors src/components/sites/AddSiteDialog.tsx: form for
// SitesController.addCustomSite() (see makeCustomSite() upstream).
//
// Popup/Dialog positioning isn't governed by normal anchoring - anchors
// on a Dialog resolve against whatever ambiguous "parent" it happened to
// inherit, which is what caused it to render cropped near the top-right
// corner instead of centered. Parenting explicitly to the window's
// Overlay.overlay and computing x/y from *its* width/height is the
// standard fix.
Dialog {
    id: root
    modal: true
    parent: Overlay.overlay
    x: Math.round((parent.width - width) / 2)
    y: Math.round((parent.height - height) / 2)
    width: 420
    title: "Add a site"

    enter: Transition {
        NumberAnimation { properties: "opacity"; from: 0; to: 1; duration: 160; easing.type: Easing.OutCubic }
        NumberAnimation { properties: "scale"; from: 0.92; to: 1; duration: 160; easing.type: Easing.OutCubic }
    }
    exit: Transition {
        NumberAnimation { properties: "opacity"; from: 1; to: 0; duration: 120; easing.type: Easing.InCubic }
        NumberAnimation { properties: "scale"; from: 1; to: 0.92; duration: 120; easing.type: Easing.InCubic }
    }

    Overlay.modal: Rectangle {
        color: "#00000099"
        Behavior on opacity { NumberAnimation { duration: 160 } }
    }

    background: Rectangle { color: Theme.surface; radius: Theme.radiusLg; border.color: Theme.border }

    header: Label {
        text: root.title
        color: Theme.ink
        font.bold: true
        font.pixelSize: 15
        padding: 16
        bottomPadding: 4
    }

    contentItem: ColumnLayout {
        spacing: 10

        Label { text: "Name"; color: Theme.inkDim; font.pixelSize: 12 }
        AppTextField { id: nameField; Layout.fillWidth: true; placeholderText: "My imageboard" }

        Label { text: "Site URL"; color: Theme.inkDim; font.pixelSize: 12 }
        AppTextField { id: originField; Layout.fillWidth: true; placeholderText: "https://example.org" }

        Label { text: "API schema"; color: Theme.inkDim; font.pixelSize: 12 }
        AppComboBox {
            id: schemaBox
            Layout.fillWidth: true
            model: ["yotsuba (4chan/vichan-style)", "lynxchan"]
        }

        Label { text: "Media layout"; color: Theme.inkDim; font.pixelSize: 12 }
        AppComboBox {
            id: layoutBox
            Layout.fillWidth: true
            model: ["board-dirs", "flat-cdn", "file-store"]
        }

        AppCheckBox { id: nsfwBox; text: "NSFW"; checked: true }

        RowLayout {
            Layout.fillWidth: true
            Layout.topMargin: 8
            Item { Layout.fillWidth: true }
            AppButton { text: "Cancel"; flat: true; onClicked: root.close() }
            AppButton {
                text: "Add site"
                checked: true
                enabled: nameField.text.length > 0 && originField.text.length > 0
                onClicked: {
                    SitesController.addCustomSite({
                        id: "custom-" + Date.now(),
                        name: nameField.text,
                        schema: schemaBox.currentIndex === 1 ? "lynxchan" : "yotsuba",
                        mediaLayout: layoutBox.currentText,
                        origin: originField.text,
                        nsfw: nsfwBox.checked,
                        postEngine: schemaBox.currentIndex === 1 ? "lynxchan" : "external",
                    })
                    root.close()
                    nameField.text = ""
                    originField.text = ""
                }
            }
        }
    }
}
