pragma Singleton
import QtQuick
import Channex

// Design-system tokens every component reads from - color, radius,
// spacing, and a couple of small helpers - so a single pass here
// reshapes the whole app's look.
//
// The dark palette is deliberately cool-tinted (a faint blue/indigo
// undertone in the greys, rather than flat neutral grey) and has more
// separation between adjacent surface steps than a typical "just
// increment the grey" scale - that's what gives stacked cards
// (Sidebar-on-canvas, a dialog-on-Sidebar, a hovered row-on-dialog)
// actual visible depth instead of reading as one flat plane.
QtObject {
    readonly property bool systemDark: Qt.styleHints.colorScheme === Qt.Dark

    readonly property bool dark: SettingsManager.theme === "light"
        ? false
        : (SettingsManager.theme === "dark" ? true : systemDark)

    readonly property color accent: SettingsManager.accentColor
    readonly property color accent2: "#8b5cf6"

    readonly property color canvas:     dark ? "#0a0b11" : "#f5f6fb"
    readonly property color surface:    dark ? "#12141c" : "#ffffff"
    readonly property color surface2:   dark ? "#171a24" : "#eef0f7"
    readonly property color surface3:   dark ? "#1e2230" : "#e3e6f0"
    readonly property color surface4:   dark ? "#262b3c" : "#d7dbe8"
    readonly property color border:     dark ? "#2b3145" : "#d5d9e6"
    readonly property color borderSoft: dark ? "#1d2130" : "#e8eaf2"
    readonly property color ink:        dark ? "#eef0f6" : "#12141d"
    readonly property color inkDim:     dark ? "#a3aac0" : "#4c5268"
    readonly property color inkFaint:   dark ? "#666f88" : "#868da5"

    readonly property color danger: "#f4415e"
    readonly property color success: "#2dd48a"

    readonly property int radiusSm: 6
    readonly property int radiusMd: 10
    readonly property int radiusLg: 16
    readonly property int radiusXl: 22

    readonly property int spacingXs: 4
    readonly property int spacingSm: 8
    readonly property int spacingMd: 12
    readonly property int spacingLg: 16
    readonly property int spacingXl: 24

    // shadowColor is the base tone every "CardShadow" layer tints with -
    // near-black even in light mode, since a shadow reads as depth
    // against either palette; components control how much of it shows
    // through via CardShadow's own layered opacities.
    readonly property color shadowColor: dark ? "#000000" : "#1a1f2e"

    readonly property var accentPresets: [
        { name: "Mint",   value: "#6ee7c9" },
        { name: "Violet", value: "#8b5cf6" },
        { name: "Sky",    value: "#38bdf8" },
        { name: "Rose",   value: "#fb7185" },
        { name: "Amber",  value: "#fbbf24" },
        { name: "Lime",   value: "#a3e635" },
    ]

    // Shorthand for a translucent variant of any color - mainly used to
    // build accent-tinted glows/washes without repeating Qt.rgba(c.r,
    // c.g, c.b, a) at every call site.
    function withAlpha(c, a) {
        return Qt.rgba(c.r, c.g, c.b, a)
    }
}
