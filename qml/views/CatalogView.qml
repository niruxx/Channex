import QtQuick
import QtQuick.Controls.Basic
import QtQuick.Layouts
import Channex
import "../components"

// Mirrors src/components/catalog/CatalogGrid.tsx.
Item {
    id: root

    readonly property var site: SitesController.findSite(NavigationController.siteId)

    CatalogController {
        id: controller
    }

    Connections {
        target: NavigationController
        function onFrameChanged() {
            if (NavigationController.view === "catalog" && NavigationController.boardCode)
                controller.load(NavigationController.siteId, NavigationController.boardCode)
        }
    }

    Component.onCompleted: {
        if (NavigationController.boardCode)
            controller.load(NavigationController.siteId, NavigationController.boardCode)
    }

    ColumnLayout {
        anchors.fill: parent
        spacing: 0

        CatalogToolbar {
            Layout.fillWidth: true
            Layout.leftMargin: 20
            Layout.rightMargin: 16
            Layout.topMargin: 16
            Layout.bottomMargin: 8
            controller: controller
        }

        Item {
            Layout.fillWidth: true
            Layout.fillHeight: true

            EmptyState {
                anchors.centerIn: parent
                visible: !NavigationController.boardCode
                iconName: "grid"
                text: "Pick a board to get started"
                supportingText: "Choose a site and a board from the navigation drawer."
            }

            AppBusyIndicator {
                anchors.centerIn: parent
                running: controller.loading && controller.count === 0
            }

            EmptyState {
                anchors.centerIn: parent
                visible: !controller.loading && controller.errorString.length > 0
                isError: true
                iconName: "alert"
                text: "Couldn't load this board"
                supportingText: controller.errorString
            }

            // Thin M3 linear indicator while refreshing over existing content.
            Rectangle {
                anchors.top: parent.top
                anchors.left: parent.left
                anchors.right: parent.right
                anchors.leftMargin: 20
                anchors.rightMargin: 20
                height: 4
                radius: 2
                color: Theme.secondaryContainer
                visible: controller.loading && controller.count > 0
                clip: true
                Rectangle {
                    id: sweepBar
                    height: parent.height
                    radius: 2
                    width: parent.width * 0.35
                    color: Theme.primary
                    NumberAnimation on x {
                        running: sweepBar.parent.visible
                        loops: Animation.Infinite
                        from: -sweepBar.width
                        to: sweepBar.parent.width
                        duration: 1100
                        easing.type: Easing.InOutCubic
                    }
                }
            }

            GridView {
                id: gridView
                anchors.fill: parent
                anchors.leftMargin: 16
                anchors.rightMargin: 8
                anchors.topMargin: 8
                // visible is intentionally NOT gated on controller.loading:
                // that used to hard-cut the view invisible for the whole
                // reload (board switch or a mode-switch that raced a
                // reload), which meant the opacity/scale Behaviors below
                // ran to completion while nothing was on screen to show
                // them - so the view would just "pop" in fully faded-in
                // the instant loading cleared, instead of visibly
                // animating. The BusyIndicator overlay communicates
                // loading instead; stale content staying up underneath it
                // is normal, non-jarring behavior.
                readonly property bool modeActive: SettingsManager.catalogViewMode !== "list"
                opacity: modeActive ? 1 : 0
                scale: modeActive ? 1 : 0.985
                visible: opacity > 0 && NavigationController.boardCode
                Behavior on opacity { NumberAnimation { duration: 180; easing.type: Easing.OutCubic } }
                Behavior on scale { NumberAnimation { duration: 180; easing.type: Easing.OutCubic } }
                model: controller
                // Cells stretch to fill the row exactly (no ragged right
                // gutter): pick a column count from the target card
                // width, then divide the available width evenly.
                readonly property int targetCell: SettingsManager.catalogViewMode === "compact" ? 168 : 236
                readonly property int columns: Math.max(1, Math.floor(width / targetCell))
                cellWidth: Math.floor(width / columns)
                // Tall enough for a full-width-square thumbnail plus subject
                // + (grid only) a 3-line excerpt + the reply/image counts
                // row without clipping into the row below - see ThreadCard.
                cellHeight: SettingsManager.catalogViewMode === "compact" ? cellWidth + 64 : cellWidth + 128
                clip: true

                populate: Transition {
                    NumberAnimation { properties: "opacity"; from: 0; to: 1; duration: 200; easing.type: Easing.OutCubic }
                    NumberAnimation { properties: "scale"; from: 0.92; to: 1; duration: 200; easing.type: Easing.OutCubic }
                }
                add: Transition {
                    NumberAnimation { properties: "opacity"; from: 0; to: 1; duration: 180; easing.type: Easing.OutCubic }
                    NumberAnimation { properties: "scale"; from: 0.92; to: 1; duration: 180; easing.type: Easing.OutCubic }
                }
                displaced: Transition {
                    NumberAnimation { properties: "x,y"; duration: 180; easing.type: Easing.OutCubic }
                }

                bottomMargin: 16
                ScrollBar.vertical: AppScrollBar {}

                delegate: Item {
                    width: gridView.cellWidth - 12
                    height: gridView.cellHeight - 12
                    ThreadCard {
                        anchors.fill: parent
                        post: model.post
                        site: root.site
                        compact: SettingsManager.catalogViewMode === "compact"
                        onOpened: NavigationController.goThread(NavigationController.siteId, NavigationController.boardCode, String(model.threadId))
                    }
                }
            }

            ListView {
                id: listView
                anchors.fill: parent
                anchors.leftMargin: 16
                anchors.rightMargin: 16
                anchors.topMargin: 8
                bottomMargin: 16
                spacing: 8
                clip: true
                readonly property bool modeActive: SettingsManager.catalogViewMode === "list"
                opacity: modeActive ? 1 : 0
                scale: modeActive ? 1 : 0.985
                visible: opacity > 0 && NavigationController.boardCode
                Behavior on opacity { NumberAnimation { duration: 180; easing.type: Easing.OutCubic } }
                Behavior on scale { NumberAnimation { duration: 180; easing.type: Easing.OutCubic } }
                model: controller

                populate: Transition {
                    NumberAnimation { properties: "opacity"; from: 0; to: 1; duration: 200; easing.type: Easing.OutCubic }
                }
                add: Transition {
                    NumberAnimation { properties: "opacity"; from: 0; to: 1; duration: 180; easing.type: Easing.OutCubic }
                }
                displaced: Transition {
                    NumberAnimation { properties: "y"; duration: 180; easing.type: Easing.OutCubic }
                }

                delegate: Item {
                    width: listView.width
                    height: 96

                    // ThreadListRow has a `required property`; using it
                    // directly as the delegate root is unreliable with Qt
                    // Quick's implicit role-to-required-property binding
                    // (post ends up permanently undefined - see ThreadView's
                    // PostCard delegate for the same issue).
                    ThreadListRow {
                        width: parent.width
                        post: model.post
                        site: root.site
                        onOpened: NavigationController.goThread(NavigationController.siteId, NavigationController.boardCode, String(model.threadId))
                    }
                }
            }
        }
    }
}
