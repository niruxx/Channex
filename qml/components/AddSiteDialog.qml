import QtQuick
import QtQuick.Controls.Basic
import QtQuick.Layouts
import Channex

// Mirrors src/components/sites/AddSiteDialog.tsx: form for
// SitesController.addCustomSite() (see makeCustomSite() upstream).
Dialog {
    id: root
    modal: true
    anchors.centerIn: parent
    width: 420
    title: "Add a site"

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
