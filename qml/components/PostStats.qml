import QtQuick
import Channex

// Reply / image counts with M3-style leading icons (labelMedium).
Row {
    property int replies: 0
    property int images: 0
    spacing: 14

    Repeater {
        model: [ { icon: "chat", n: replies }, { icon: "image", n: images } ]
        delegate: Row {
            spacing: 5
            AppIcon { anchors.verticalCenter: parent.verticalCenter; name: modelData.icon; iconSize: 14; color: Theme.onSurfaceVariantColor }
            Text {
                anchors.verticalCenter: parent.verticalCenter
                text: modelData.n
                color: Theme.onSurfaceVariantColor
                font.pixelSize: Theme.labelMedium
                font.weight: Font.Medium
            }
        }
    }
}
