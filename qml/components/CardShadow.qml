import QtQuick
import Channex

// A minimal, crisp drop-shadow for genuinely floating/elevated
// surfaces only - dialogs, popups, context menus - not structural
// chrome that's always on screen (Sidebar/TitleBar are flat, no
// shadow, by design). One low-opacity, downward-offset rectangle
// stands in for a real gaussian blur: Qt's MultiEffect proved
// unreliable for colorization on this Qt/RHI setup earlier this
// session, and a proper blur carries the same risk, so this favors
// "flat and correct" over "soft and might render broken."
//
// Usage: drop as the first child of the surface (so it paints
// behind), anchored/fill-parented to it, with `radius` matching the
// surface's own radius.
Item {
    id: root
    property int radius: Theme.radiusLg
    z: -1

    Rectangle {
        anchors.fill: parent
        anchors.topMargin: 4
        radius: root.radius
        color: Theme.withAlpha(Theme.shadowColor, 0.18)
    }
}
