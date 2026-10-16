import QtQuick
import QtQuick.Controls.Basic
import QtQuick.Layouts
import Channex

// M3 basic dialog: surfaceContainerHigh, 28px corners, 24px padding,
// headlineSmall title (+ optional supporting text), 32% scrim, and the
// emphasized-decelerate scale/fade entrance.
//
// Parented to Overlay.overlay and centered from *its* size - anchors on
// a Popup resolve against whatever ambiguous parent it inherited, which
// once rendered dialogs cropped into a corner.
Dialog {
    id: root
    property string supportingText: ""

    modal: true
    parent: Overlay.overlay
    x: Math.round((parent.width - width) / 2)
    y: Math.round((parent.height - height) / 2)
    padding: 24
    topPadding: 16

    enter: Transition {
        NumberAnimation { property: "opacity"; from: 0; to: 1; duration: Theme.durMedium; easing.type: Easing.BezierSpline; easing.bezierCurve: Theme.easeEmphasizedDecel }
        NumberAnimation { property: "scale"; from: 0.9; to: 1; duration: Theme.durMedium; easing.type: Easing.BezierSpline; easing.bezierCurve: Theme.easeEmphasizedDecel }
    }
    exit: Transition {
        NumberAnimation { property: "opacity"; from: 1; to: 0; duration: Theme.durShort; easing.type: Easing.BezierSpline; easing.bezierCurve: Theme.easeEmphasizedAccel }
        NumberAnimation { property: "scale"; from: 1; to: 0.95; duration: Theme.durShort; easing.type: Easing.BezierSpline; easing.bezierCurve: Theme.easeEmphasizedAccel }
    }

    Overlay.modal: Rectangle {
        color: Theme.withAlpha(Theme.scrim, 0.32)
    }

    background: Rectangle {
        color: Theme.surfaceContainerHigh
        radius: Theme.shapeExtraLarge
        CardShadow { radius: parent.radius; level: 3 }
    }

    header: ColumnLayout {
        spacing: 12
        Text {
            Layout.fillWidth: true
            Layout.leftMargin: 24
            Layout.rightMargin: 24
            Layout.topMargin: 24
            text: root.title
            color: Theme.onSurfaceColor
            font.pixelSize: Theme.headlineSmall
            wrapMode: Text.WordWrap
        }
        Text {
            visible: root.supportingText.length > 0
            Layout.fillWidth: true
            Layout.leftMargin: 24
            Layout.rightMargin: 24
            text: root.supportingText
            color: Theme.onSurfaceVariantColor
            font.pixelSize: Theme.bodyMedium
            wrapMode: Text.WordWrap
        }
    }
}
