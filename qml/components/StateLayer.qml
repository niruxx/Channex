import QtQuick
import Channex

// M3 state layer: a translucent overlay in the component's content
// color that signals hover / focus / press. Drop it inside any
// interactive surface, after the surface's own fill, and bind
// `hovered` / `pressed` from that component's handlers.
Rectangle {
    id: root
    property bool hovered: false
    property bool pressed: false
    property bool focused: false
    property color layerColor: Theme.onSurfaceColor

    anchors.fill: parent
    radius: parent && parent.radius !== undefined ? parent.radius : 0
    color: layerColor
    opacity: pressed ? Theme.pressedOpacity
           : focused ? Theme.focusOpacity
           : hovered ? Theme.hoverOpacity : 0

    Behavior on opacity { NumberAnimation { duration: Theme.durShort; easing.type: Easing.BezierSpline; easing.bezierCurve: Theme.easeStandard } }
}
