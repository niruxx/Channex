import QtQuick
import QtQuick.Controls.Basic
import Channex

// Mirrors src/components/sites/BoardList.tsx: live-discovered board
// list for the active site (falls back to ChanSite.defaultBoards -
// see SitesController::loadBoards / mergeDefaultBoards).
ListView {
    id: root
    clip: true
    model: SitesController.currentBoards
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

    header: Column {
        width: root.width
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
            onClicked: NavigationController.goCatalog(NavigationController.siteId, modelData.code)
        }
    }
}
