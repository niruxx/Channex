import QtQuick
import Channex

// An image with rounded corners. Qt Quick's `clip` only clips to the
// bounding rectangle, and MultiEffect masking proved unreliable on this
// Qt/RHI setup, so the corners are instead covered by a ring in
// `maskColor` - which must match the solid surface the image sits on.
Item {
    id: root
    property alias source: img.source
    property alias status: img.status
    property alias fillMode: img.fillMode
    property int radius: Theme.shapeMedium
    property color maskColor: Theme.surface
    property color placeholderColor: Theme.surfaceContainerHighest
    property bool hideImage: false
    readonly property int ring: Math.max(4, radius)

    clip: true

    Rectangle {
        anchors.fill: parent
        radius: root.radius
        color: root.placeholderColor
    }

    Image {
        id: img
        anchors.fill: parent
        fillMode: Image.PreserveAspectCrop
        asynchronous: true
        visible: !root.hideImage
        opacity: status === Image.Ready ? 1 : 0
        Behavior on opacity { NumberAnimation { duration: Theme.durMedium } }
    }

    Rectangle {
        visible: root.radius > 0
        anchors.fill: parent
        anchors.margins: -root.ring
        radius: root.radius + root.ring
        color: "transparent"
        border.width: root.ring
        border.color: root.maskColor
    }
}
