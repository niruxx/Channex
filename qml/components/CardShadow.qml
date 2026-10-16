import QtQuick
import Channex

// M3 elevation shadow for floating surfaces (menus, dialogs, FAB-like
// buttons, hovered cards). A real gaussian blur via MultiEffect proved
// unreliable on this Qt/RHI setup, so the soft edge is approximated by
// a few stacked, progressively larger, very low-alpha rounded rects -
// a "key" shadow offset downward plus a faint ambient spread.
//
// Usage: first child of the surface (paints behind it via z: -1),
// with `radius` matching the surface's radius. `level` follows M3's
// elevation levels 0-5.
Item {
    id: root
    property int radius: Theme.shapeLarge
    property int level: 2
    anchors.fill: parent
    z: -1
    visible: level > 0

    readonly property real strength: Theme.dark ? 0.32 : 0.11

    Repeater {
        model: Math.min(root.level, 5) + 1
        delegate: Rectangle {
            readonly property int spread: index
            x: -spread
            y: -spread + Math.ceil(root.level * 0.75)
            width: root.width + spread * 2
            height: root.height + spread * 2
            radius: root.radius + spread
            color: Theme.withAlpha(Theme.shadow, root.strength / (index + 1.5))
        }
    }
}
