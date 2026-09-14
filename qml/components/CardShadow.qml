import QtQuick
import Channex

// A cheap drop-shadow illusion for floating cards (Sidebar, TitleBar,
// Settings cards, dialogs) - three progressively larger, lower-opacity,
// downward-offset rectangles standing in for a real gaussian blur.
// Qt's MultiEffect (QtQuick.Effects) turned out unreliable for
// colorization on this Qt/RHI setup earlier this session, and a proper
// blur carries the same risk; layered flat rectangles are less
// beautiful up close but can't render broken, which matters more when
// this can't be visually spot-checked before shipping.
//
// Usage: drop as the first child of the card (so it paints behind),
// anchored/fill-parented to the card, with `radius` matching the
// card's own radius.
Item {
    id: root
    property int radius: Theme.radiusLg
    z: -1

    Rectangle {
        anchors.fill: parent
        anchors.topMargin: 10
        anchors.margins: -6
        radius: root.radius + 4
        color: Theme.withAlpha(Theme.shadowColor, 0.10)
    }
    Rectangle {
        anchors.fill: parent
        anchors.topMargin: 5
        anchors.margins: -2
        radius: root.radius + 2
        color: Theme.withAlpha(Theme.shadowColor, 0.14)
    }
    Rectangle {
        anchors.fill: parent
        anchors.topMargin: 2
        radius: root.radius
        color: Theme.withAlpha(Theme.shadowColor, 0.16)
    }
}
