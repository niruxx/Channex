import QtQuick
import QtQuick.Controls.Basic
import Channex

// M3 menu container: surfaceContainer, 4px corners, level-2 shadow,
// 8px vertical padding. Fill with AppMenuItem.
Menu {
    id: control
    topPadding: 8
    bottomPadding: 8

    enter: Transition {
        NumberAnimation { property: "opacity"; from: 0; to: 1; duration: Theme.durShort; easing.type: Easing.BezierSpline; easing.bezierCurve: Theme.easeEmphasizedDecel }
        NumberAnimation { property: "scale"; from: 0.92; to: 1; duration: Theme.durShort; easing.type: Easing.BezierSpline; easing.bezierCurve: Theme.easeEmphasizedDecel }
    }
    exit: Transition {
        NumberAnimation { property: "opacity"; from: 1; to: 0; duration: 100; easing.type: Easing.BezierSpline; easing.bezierCurve: Theme.easeEmphasizedAccel }
    }

    background: Rectangle {
        implicitWidth: 200
        color: Theme.surfaceContainer
        radius: Theme.shapeExtraSmall
        CardShadow { radius: parent.radius; level: 2 }
    }
}
