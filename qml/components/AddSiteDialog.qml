import QtQuick
import QtQuick.Controls.Basic
import QtQuick.Layouts
import Channex

// Form for SitesController.addCustomSite() (see makeCustomSite()
// upstream), as an M3 dialog with filled fields.
AppDialog {
    id: root
    width: 460
    title: "Add a site"
    supportingText: "Any 4chan-API-compatible (vichan, etc.) or LynxChan imageboard."

    contentItem: ColumnLayout {
        spacing: 16

        AppTextField { id: nameField; Layout.fillWidth: true; label: "Name"; placeholderText: "My imageboard" }
        AppTextField { id: originField; Layout.fillWidth: true; label: "Site URL"; placeholderText: "https://example.org" }

        AppComboBox {
            id: schemaBox
            Layout.fillWidth: true
            label: "API schema"
            model: ["yotsuba (4chan/vichan-style)", "lynxchan"]
        }

        AppComboBox {
            id: layoutBox
            Layout.fillWidth: true
            label: "Media layout"
            model: ["board-dirs", "flat-cdn", "file-store"]
        }

        AppSwitch { id: nsfwBox; Layout.fillWidth: true; text: "NSFW site"; checked: true }

        RowLayout {
            Layout.fillWidth: true
            Layout.topMargin: 8
            spacing: 8
            Item { Layout.fillWidth: true }
            AppButton { variant: "text"; text: "Cancel"; onClicked: root.close() }
            AppButton {
                variant: "filled"
                text: "Add site"
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
