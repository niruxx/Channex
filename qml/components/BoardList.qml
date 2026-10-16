import QtQuick
import QtQuick.Controls.Basic
import Channex

// Live-discovered board list for the active site (falls back to
// ChanSite.defaultBoards - see SitesController::loadBoards /
// mergeDefaultBoards), as M3 navigation drawer items.
ListView {
    id: root
    clip: true
    boundsBehavior: Flickable.StopAtBounds
    ScrollBar.vertical: AppScrollBar {}

    // Reads SettingsManager.hiddenBoards directly (a real NOTIFYing
    // property) rather than through a per-board invokable lookup, so
    // this recomputes correctly whenever the Settings "Boards" section
    // hides/shows something - see SettingsManager::setBoardHidden.
    readonly property var hiddenForSite: SettingsManager.hiddenBoards[NavigationController.siteId] || []
    model: SitesController.currentBoards.filter(function (b) { return hiddenForSite.indexOf(b.code) === -1 })
    spacing: 0

    populate: Transition {
        NumberAnimation { properties: "opacity"; from: 0; to: 1; duration: Theme.durMedium; easing.type: Easing.BezierSpline; easing.bezierCurve: Theme.easeStandard }
    }
    add: Transition {
        NumberAnimation { properties: "opacity"; from: 0; to: 1; duration: Theme.durShort }
    }
    displaced: Transition {
        NumberAnimation { properties: "y"; duration: Theme.durShort; easing.type: Easing.BezierSpline; easing.bezierCurve: Theme.easeStandard }
    }

    // Height collapses to 0 when not loading - visible:false alone would
    // still reserve the header's implicit height as a gap.
    header: Item {
        width: root.width
        height: SitesController.boardsLoading ? 40 : 0
        clip: true
        visible: SitesController.boardsLoading
        Row {
            anchors.verticalCenter: parent.verticalCenter
            x: 16
            spacing: 10
            AppBusyIndicator { width: 18; height: 18; lineWidth: 2.5; running: SitesController.boardsLoading }
            Text { text: "Loading boards…"; color: Theme.onSurfaceVariantColor; font.pixelSize: Theme.bodySmall; anchors.verticalCenter: parent.verticalCenter }
        }
    }

    delegate: NavItem {
        id: boardDelegate
        width: root.width - 6
        text: "/" + modelData.code + "/"
        secondaryText: modelData.title || ""
        selected: modelData.code === NavigationController.boardCode && NavigationController.view !== "bookmarks"
                  && NavigationController.view !== "downloads" && NavigationController.view !== "settings"
        onClicked: NavigationController.goCatalog(NavigationController.siteId, modelData.code)
        onRightClicked: boardContextMenu.popup()

        AppMenu {
            id: boardContextMenu
            AppMenuItem {
                text: "Hide from sidebar"
                iconName: "x"
                onTriggered: SettingsManager.setBoardHidden(NavigationController.siteId, modelData.code, true)
            }
        }
    }
}
