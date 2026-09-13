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
                visible: NavigationController.boardCode && SettingsManager.catalogViewMode !== "list" && !controller.loading
                model: controller
                cellWidth: SettingsManager.catalogViewMode === "compact" ? 160 : 232
                cellHeight: SettingsManager.catalogViewMode === "compact" ? 190 : 300
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
                visible: NavigationController.boardCode && SettingsManager.catalogViewMode === "list" && !controller.loading
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
