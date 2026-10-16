import QtQuick
import QtQuick.Controls.Basic
import QtQuick.Layouts
import Channex

// Site avatars, hiding sites flagged nsfw when hideNsfwSites is on
// (unless it's the active one), with a "+N" reveal toggle and an "Add
// site" affordance opening AddSiteDialog.
//
// M3-expressive shape morph: avatars are circles, and the selected one
// morphs to a rounded square in primaryContainer - the same selection
// language as M3's FAB / navigation-rail indicators.
ColumnLayout {
    id: root
    spacing: 8

    property bool revealHidden: false

    readonly property var visibleSites: SitesController.sites.filter(function (s) {
        return revealHidden || !s.nsfw || !SettingsManager.hideNsfwSites || s.id === NavigationController.siteId
    })
    readonly property int hiddenCount: SitesController.sites.length - visibleSites.length

    Flow {
        Layout.fillWidth: true
        spacing: 8

        add: Transition {
            NumberAnimation { properties: "opacity"; from: 0; to: 1; duration: Theme.durShort; easing.type: Easing.BezierSpline; easing.bezierCurve: Theme.easeStandard }
            NumberAnimation { properties: "scale"; from: 0.85; to: 1; duration: Theme.durShort; easing.type: Easing.BezierSpline; easing.bezierCurve: Theme.easeEmphasizedDecel }
        }
        move: Transition {
            NumberAnimation { properties: "x,y"; duration: Theme.durShort; easing.type: Easing.BezierSpline; easing.bezierCurve: Theme.easeStandard }
        }

        Repeater {
            model: root.visibleSites
            delegate: Rectangle {
                id: siteAvatar
                width: 44; height: 44
                readonly property bool current: modelData.id === NavigationController.siteId
                readonly property bool hasFavicon: !!modelData.favicon && modelData.favicon.length > 0
                radius: current ? Theme.shapeMedium : width / 2
                color: current ? Theme.primaryContainer : Theme.surfaceContainerHighest
                clip: true

                Behavior on radius { NumberAnimation { duration: Theme.durMedium; easing.type: Easing.BezierSpline; easing.bezierCurve: Theme.easeEmphasizedDecel } }
                Behavior on color { ColorAnimation { duration: Theme.durShort } }

                StateLayer {
                    layerColor: siteAvatar.current ? Theme.onPrimaryContainerColor : Theme.onSurfaceColor
                    hovered: siteHover.hovered
                    pressed: siteMouse.pressed
                }

                HoverHandler { id: siteHover }

                Text {
                    anchors.centerIn: parent
                    visible: !siteAvatar.hasFavicon || favicon.status !== Image.Ready
                    text: modelData.name.charAt(0).toUpperCase()
                    color: siteAvatar.current ? Theme.onPrimaryContainerColor : Theme.onSurfaceVariantColor
                    font.pixelSize: Theme.titleMedium
                    font.weight: Font.Medium
                }

                Image {
                    id: favicon
                    anchors.centerIn: parent
                    width: 22; height: 22
                    visible: siteAvatar.hasFavicon && status === Image.Ready
                    source: siteAvatar.hasFavicon ? modelData.favicon : ""
                    fillMode: Image.PreserveAspectFit
                    asynchronous: true
                    cache: true
                }

                MouseArea {
                    id: siteMouse
                    anchors.fill: parent
                    cursorShape: Qt.PointingHandCursor
                    onClicked: {
                        SitesController.currentSiteId = modelData.id
                        NavigationController.setSite(modelData.id)
                    }
                }

                ToolTip {
                    visible: siteHover.hovered
                    delay: 500
                    text: modelData.name
                    padding: 8; topPadding: 4; bottomPadding: 4
                    contentItem: Text { text: modelData.name; color: Theme.inverseOnSurface; font.pixelSize: Theme.bodySmall }
                    background: Rectangle { radius: Theme.shapeExtraSmall; color: Theme.inverseSurface }
                }
            }
        }

        Rectangle {
            width: 44; height: 44; radius: 22
            color: "transparent"
            border.width: 1
            border.color: Theme.outlineVariant
            visible: root.hiddenCount > 0 || root.revealHidden
            StateLayer { hovered: hiddenHover.hovered }
            HoverHandler { id: hiddenHover }
            Text {
                anchors.centerIn: parent
                text: root.revealHidden ? "−" : "+" + root.hiddenCount
                color: Theme.onSurfaceVariantColor
                font.pixelSize: Theme.labelMedium
                font.weight: Font.Medium
            }
            MouseArea { anchors.fill: parent; cursorShape: Qt.PointingHandCursor; onClicked: root.revealHidden = !root.revealHidden }
        }

        Rectangle {
            width: 44; height: 44; radius: 22
            color: "transparent"
            border.width: 1
            border.color: Theme.outlineVariant
            StateLayer { hovered: addHover.hovered }
            HoverHandler { id: addHover }
            AppIcon { anchors.centerIn: parent; name: "plus"; iconSize: 20; color: Theme.primary }
            MouseArea { anchors.fill: parent; cursorShape: Qt.PointingHandCursor; onClicked: addSiteDialog.open() }
        }
    }

    AddSiteDialog { id: addSiteDialog }
}
