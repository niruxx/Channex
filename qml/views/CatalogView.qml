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

        Rectangle {
            Layout.fillWidth: true
            height: 52
            color: "transparent"
            CatalogToolbar { anchors.fill: parent; anchors.margins: 8; controller: controller }
        }

        Item {
            Layout.fillWidth: true
            Layout.fillHeight: true

            EmptyState {
                anchors.centerIn: parent
                visible: !NavigationController.boardCode
                text: "Pick a board to get started"
            }

            BusyIndicator {
                anchors.centerIn: parent
                running: controller.loading
                visible: controller.loading && controller.count === 0
            }

            Text {
                anchors.centerIn: parent
                visible: !controller.loading && controller.errorString.length > 0
                text: "Couldn't load this board: " + controller.errorString
                color: Theme.danger
            }

            GridView {
                id: gridView
                anchors.fill: parent
                anchors.margins: 12
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
                cellWidth: SettingsManager.catalogViewMode === "compact" ? 160 : 232
                // Tall enough for a full-width-square thumbnail plus subject
                // + (grid only) a 3-line excerpt + the reply/image counts
                // row without clipping into the row below - see ThreadCard.
                cellHeight: SettingsManager.catalogViewMode === "compact" ? 210 : 340
                clip: true

                delegate: Item {
                    width: gridView.cellWidth - 8
                    height: gridView.cellHeight - 8
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
                anchors.margins: 12
                spacing: 8
                clip: true
                readonly property bool modeActive: SettingsManager.catalogViewMode === "list"
                opacity: modeActive ? 1 : 0
                scale: modeActive ? 1 : 0.985
                visible: opacity > 0 && NavigationController.boardCode
                Behavior on opacity { NumberAnimation { duration: 180; easing.type: Easing.OutCubic } }
                Behavior on scale { NumberAnimation { duration: 180; easing.type: Easing.OutCubic } }
                model: controller

                delegate: Item {
                    width: listView.width
                    height: 88

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
