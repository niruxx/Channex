import QtQuick
import QtQuick.Controls.Basic
import Channex

// Mirrors src/components/sites/BoardList.tsx: live-discovered board
// list for the active site (falls back to ChanSite.defaultBoards -
// see SitesController::loadBoards / mergeDefaultBoards).
ListView {
    id: root
    clip: true

    // Reads SettingsManager.hiddenBoards directly (a real NOTIFYing
    // property) rather than through a per-board invokable lookup, so
    // this recomputes correctly whenever the Settings "Boards" section
    // hides/shows something - see SettingsManager::setBoardHidden.
    readonly property var hiddenForSite: SettingsManager.hiddenBoards[NavigationController.siteId] || []
    model: SitesController.currentBoards.filter(function (b) { return hiddenForSite.indexOf(b.code) === -1 })
    spacing: 2

    populate: Transition {
        NumberAnimation { properties: "opacity"; from: 0; to: 1; duration: 180; easing.type: Easing.OutCubic }
    }
    add: Transition {
        NumberAnimation { properties: "opacity"; from: 0; to: 1; duration: 160; easing.type: Easing.OutCubic }
    }
    displaced: Transition {
        NumberAnimation { properties: "y"; duration: 160; easing.type: Easing.OutCubic }
    }

    // visible:false alone only hides the paint - the header still
    // reserved its full implicit height (padded text, ~32px) in the
    // list's layout even while not loading, showing up as a persistent
    // gap between the site switcher and the first board. Collapsing
    // height to 0 when hidden actually removes that reserved space.
    header: Column {
        width: root.width
        height: SitesController.boardsLoading ? implicitHeight : 0
        clip: true
        visible: SitesController.boardsLoading
        Text {
            text: "Loading boards…"
            color: Theme.inkFaint
            font.pixelSize: 12
            padding: 8
        }
    }

    delegate: Rectangle {
        id: boardDelegate
        width: root.width
        height: 32
        radius: Theme.radiusSm
        readonly property bool current: modelData.code === NavigationController.boardCode
        color: current ? Theme.surface3 : (hoverHandler.hovered ? Theme.surface2 : "transparent")

        Behavior on color { ColorAnimation { duration: 120; easing.type: Easing.OutCubic } }

        Text {
            anchors.verticalCenter: parent.verticalCenter
            anchors.left: parent.left
            anchors.leftMargin: 10
            text: "/" + modelData.code + "/  " + (modelData.title || "")
            color: boardDelegate.current ? Theme.ink : Theme.inkDim
            font.pixelSize: 13
            elide: Text.ElideRight
            width: parent.width - 20

            Behavior on color { ColorAnimation { duration: 120 } }
        }

        HoverHandler { id: hoverHandler }

        MouseArea {
            anchors.fill: parent
            cursorShape: Qt.PointingHandCursor
            acceptedButtons: Qt.LeftButton | Qt.RightButton
            onClicked: (mouse) => {
                if (mouse.button === Qt.RightButton) boardContextMenu.popup()
                else NavigationController.goCatalog(NavigationController.siteId, modelData.code)
            }
        }

        Menu {
            id: boardContextMenu

            enter: Transition {
                NumberAnimation { properties: "opacity"; from: 0; to: 1; duration: 120; easing.type: Easing.OutCubic }
            }
            exit: Transition {
                NumberAnimation { properties: "opacity"; from: 1; to: 0; duration: 90; easing.type: Easing.InCubic }
            }

            background: Rectangle {
                implicitWidth: 190
                color: Theme.surface2
                border.width: 1
                border.color: Theme.border
                radius: Theme.radiusSm
                CardShadow { anchors.fill: parent; radius: Theme.radiusSm }
            }

            MenuItem {
                text: "Hide from sidebar"
                onTriggered: SettingsManager.setBoardHidden(NavigationController.siteId, modelData.code, true)

                contentItem: Text {
                    text: parent.text
                    color: Theme.ink
                    font.pixelSize: 13
                    verticalAlignment: Text.AlignVCenter
                    leftPadding: 8
                    rightPadding: 8
                }
                background: Rectangle {
                    implicitHeight: 30
                    radius: Theme.radiusSm
                    color: parent.hovered ? Theme.surface3 : "transparent"
                }
            }
        }
    }
}
