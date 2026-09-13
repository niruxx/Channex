import QtQuick
import QtQuick.Controls.Basic
import QtQuick.Layouts
import Channex

// Mirrors src/components/sites/SiteSwitcher.tsx: a row of site avatars,
// hides sites flagged nsfw when hideNsfwSites is on (unless it's the
// currently active site), with a "+N hidden" reveal toggle and an "Add
// site" affordance opening AddSiteDialog.
ColumnLayout {
    spacing: 8

    property bool revealHidden: false

    readonly property var visibleSites: SitesController.sites.filter(function (s) {
        return revealHidden || !s.nsfw || !SettingsManager.hideNsfwSites || s.id === NavigationController.siteId
    })
    readonly property int hiddenCount: SitesController.sites.length - visibleSites.length

    Flow {
        Layout.fillWidth: true
        spacing: 8

        Repeater {
            model: visibleSites
            delegate: Rectangle {
                width: 40; height: 40; radius: Theme.radiusMd
                readonly property bool current: modelData.id === NavigationController.siteId
                color: current ? Theme.surface3 : (siteHover.hovered ? Theme.surface3 : Theme.surface2)
                border.width: current ? 2 : 0
                border.color: modelData.accent
                clip: true
                scale: siteHover.hovered && !current ? 1.06 : 1.0

                Behavior on color { ColorAnimation { duration: 120; easing.type: Easing.OutCubic } }
                Behavior on scale { NumberAnimation { duration: 120; easing.type: Easing.OutCubic } }

                readonly property bool hasFavicon: !!modelData.favicon && modelData.favicon.length > 0

                HoverHandler { id: siteHover }

                Text {
                    anchors.centerIn: parent
                    visible: !hasFavicon || favicon.status !== Image.Ready
                    text: modelData.name.charAt(0).toUpperCase()
                    color: modelData.accent
                    font.bold: true
                }

                Image {
                    id: favicon
                    anchors.centerIn: parent
                    width: 22; height: 22
                    visible: hasFavicon && status === Image.Ready
                    source: hasFavicon ? modelData.favicon : ""
                    fillMode: Image.PreserveAspectFit
                    asynchronous: true
                    cache: true
                }

                MouseArea {
                    anchors.fill: parent
                    cursorShape: Qt.PointingHandCursor
                    onClicked: {
                        SitesController.currentSiteId = modelData.id
                        NavigationController.setSite(modelData.id)
                    }
                }
            }
        }

        Rectangle {
            width: 40; height: 40; radius: Theme.radiusMd
            color: "transparent"
            border.width: 1
            border.color: Theme.border
            visible: hiddenCount > 0
            Text { anchors.centerIn: parent; text: "+" + hiddenCount; color: Theme.inkFaint; font.pixelSize: 11 }
            MouseArea { anchors.fill: parent; onClicked: revealHidden = !revealHidden }
        }

        Rectangle {
            width: 40; height: 40; radius: Theme.radiusMd
            color: "transparent"
            border.width: 1
            border.color: Theme.border
            Text { anchors.centerIn: parent; text: "+"; color: Theme.inkDim; font.pixelSize: 16 }
            MouseArea { anchors.fill: parent; onClicked: addSiteDialog.open() }
        }
    }

    AddSiteDialog { id: addSiteDialog }
}
