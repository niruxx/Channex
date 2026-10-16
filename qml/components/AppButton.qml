import QtQuick
import QtQuick.Controls.Basic
import Channex

// M3 button family in one control. Pick the style with `variant`:
//
//   "filled"   - highest emphasis (primary action of a screen/dialog)
//   "tonal"    - filled tonal (secondaryContainer), medium emphasis
//   "outlined" - medium-low emphasis
//   "text"     - lowest emphasis
//   "elevated" - tonal surface + shadow, for use over busy content
//   "icon" / "filledIcon" / "tonalIcon" / "outlinedIcon"
//              - the four M3 icon-button styles (set iconName, no text)
//
// Left unset, `variant` is inferred from the older flat/checked props
// so any call site that predates the M3 pass still renders sensibly.
// Icon buttons treat `checked` as the M3 toggle "selected" state.
// `onMedia` re-tints for use over photos/video (the lightbox), where
// theme surface colors would vanish against the content.
Button {
    id: control

    property string iconName: ""
    property string variant: ""
    property bool onMedia: false
    property bool compact: false
    property string tooltip: ""

    readonly property bool iconOnly: iconName.length > 0 && text.length === 0
    readonly property bool hasIcon: iconName.length > 0

    readonly property string resolvedVariant: variant.length > 0 ? variant
        : iconOnly ? (flat ? "icon" : "tonalIcon")
        : flat ? (checked && !checkable ? "filled" : "text")
        : (checked && !checkable ? "filled" : "tonal")

    readonly property bool selected: checkable && checked

    readonly property color containerColor: {
        if (onMedia) {
            switch (resolvedVariant) {
            case "filled": case "filledIcon": return Theme.primary
            case "tonal": case "tonalIcon": case "elevated": return Qt.rgba(0, 0, 0, 0.55)
            default: return "transparent"
            }
        }
        switch (resolvedVariant) {
        case "filled": return Theme.primary
        case "tonal": return Theme.secondaryContainer
        case "elevated": return Theme.surfaceContainerLow
        case "filledIcon": return (checkable && !checked) ? Theme.surfaceContainerHighest : Theme.primary
        case "tonalIcon": return (checkable && !checked) ? Theme.surfaceContainerHighest : Theme.secondaryContainer
        case "outlinedIcon": return selected ? Theme.inverseSurface : "transparent"
        case "icon": return "transparent"
        default: return "transparent"
        }
    }

    readonly property color contentColor: {
        if (onMedia) {
            switch (resolvedVariant) {
            case "filled": case "filledIcon": return Theme.onPrimaryColor
            default: return "#ffffff"
            }
        }
        switch (resolvedVariant) {
        case "filled": return Theme.onPrimaryColor
        case "tonal": return Theme.onSecondaryContainerColor
        case "outlined": case "text": case "elevated": return Theme.primary
        case "filledIcon": return (checkable && !checked) ? Theme.primary : Theme.onPrimaryColor
        case "tonalIcon": return (checkable && !checked) ? Theme.onSurfaceVariantColor : Theme.onSecondaryContainerColor
        case "outlinedIcon": return selected ? Theme.inverseOnSurface : Theme.onSurfaceVariantColor
        case "icon": return selected ? Theme.primary : Theme.onSurfaceVariantColor
        default: return Theme.primary
        }
    }

    readonly property bool hasOutline: resolvedVariant === "outlined" || (resolvedVariant === "outlinedIcon" && !selected)
    readonly property int size: compact ? 32 : 40

    implicitHeight: size
    implicitWidth: iconOnly ? size : Math.max(compact ? 48 : 64, implicitContentWidth + leftPadding + rightPadding)
    leftPadding: iconOnly ? 0 : (resolvedVariant === "text" ? 12 : (hasIcon ? 16 : 24))
    rightPadding: iconOnly ? 0 : (resolvedVariant === "text" ? (hasIcon ? 16 : 12) : 24)
    topPadding: 0
    bottomPadding: 0
    font.pixelSize: compact ? Theme.labelMedium : Theme.labelLarge
    font.weight: Font.Medium
    hoverEnabled: true

    background: Rectangle {
        radius: height / 2
        color: !control.enabled
            ? (control.containerColor.a > 0 ? Theme.withAlpha(Theme.onSurfaceColor, Theme.disabledContainerOpacity) : "transparent")
            : control.containerColor
        border.width: control.hasOutline ? 1 : 0
        border.color: control.enabled ? (control.onMedia ? "#ffffff88" : Theme.outline)
                                      : Theme.withAlpha(Theme.onSurfaceColor, Theme.disabledContainerOpacity)

        Behavior on color { ColorAnimation { duration: Theme.durShort } }

        CardShadow {
            radius: parent.radius
            level: control.resolvedVariant === "elevated" && control.enabled ? (control.hovered ? 2 : 1) : 0
        }

        StateLayer {
            layerColor: control.contentColor
            hovered: control.hovered && control.enabled
            pressed: control.down
            focused: control.visualFocus
        }
    }

    contentItem: Item {
        implicitWidth: row.implicitWidth
        implicitHeight: row.implicitHeight

        Row {
            id: row
            anchors.centerIn: parent
            spacing: 8

            AppIcon {
                visible: control.hasIcon
                anchors.verticalCenter: parent.verticalCenter
                name: control.iconName
                iconSize: control.compact ? 16 : (control.iconOnly ? 20 : 18)
                color: control.enabled ? control.contentColor : Theme.withAlpha(Theme.onSurfaceColor, Theme.disabledContentOpacity)
            }

            Text {
                visible: control.text.length > 0
                anchors.verticalCenter: parent.verticalCenter
                text: control.text
                font: control.font
                color: control.enabled ? control.contentColor : Theme.withAlpha(Theme.onSurfaceColor, Theme.disabledContentOpacity)
                elide: Text.ElideRight
            }
        }
    }

    // M3 plain tooltip - mostly for icon-only buttons.
    ToolTip {
        visible: control.tooltip.length > 0 && control.hovered
        delay: 600
        text: control.tooltip
        padding: 8
        topPadding: 4
        bottomPadding: 4
        contentItem: Text { text: control.tooltip; color: Theme.inverseOnSurface; font.pixelSize: Theme.bodySmall }
        background: Rectangle { radius: Theme.shapeExtraSmall; color: Theme.inverseSurface }
    }
}
