import QtQuick
import QtQuick.Controls.Basic
import Channex

// Thin M3-tinted scrollbar. Replacing the Basic style's contentItem
// also drops its show-only-while-scrolling behavior, so that is
// restored here: the handle fades in while the view is moving or the
// bar is hovered/pressed, and never shows when everything fits.
ScrollBar {
    id: control
    hoverEnabled: true
    padding: 2

    contentItem: Rectangle {
        implicitWidth: control.hovered || control.pressed ? 8 : 6
        implicitHeight: implicitWidth
        radius: width / 2
        color: Theme.withAlpha(Theme.onSurfaceVariantColor, control.pressed ? 0.7 : 0.45)
        opacity: control.size < 1.0 && (control.active || control.hovered) ? 1 : 0
        Behavior on opacity { NumberAnimation { duration: Theme.durMedium } }
        Behavior on implicitWidth { NumberAnimation { duration: Theme.durShort } }
    }
}
