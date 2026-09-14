pragma Singleton
import QtQuick
import Channex

// Design-system tokens every component reads from - color, radius,
// spacing, and a couple of small helpers.
//
// SettingsManager.theme selects either "dark" / "light" / "system"
// (the neutral, user-accent-driven palettes) or one of five classic
// imageboard skins - "yotsuba" / "yotsubab" / "futaba" / "photon" /
// "burichan" - modeled after 4chan's own theme-selector palettes
// (footer of the site). Selecting one of those also switches `accent`
// to that skin's traditional link/highlight color instead of the
// user's chosen accentColor, since the point of picking "Yotsuba" is
// the whole authentic package, not just the background tint - hex
// values here are a close-read approximation of the real site's CSS,
// not pixel-verified against it.
QtObject {
    readonly property bool systemDark: Qt.styleHints.colorScheme === Qt.Dark

    readonly property var chanPalettes: ({
        yotsuba: {
            canvas: "#ffffee", surface: "#fffff2", surface2: "#f0e0d6", surface3: "#e9d3c7", surface4: "#dfc3b5",
            border: "#d9bfb7", borderSoft: "#e8d5cb", ink: "#000000", inkDim: "#4a3c35", inkFaint: "#87766c",
            accent: "#34345c"
        },
        yotsubab: {
            canvas: "#eef2ff", surface: "#f4f7ff", surface2: "#d6daf0", surface3: "#c7cce6", surface4: "#b7bfdb",
            border: "#b7c5d9", borderSoft: "#d3daea", ink: "#000000", inkDim: "#38405a", inkFaint: "#767ea0",
            accent: "#34345c"
        },
        futaba: {
            canvas: "#ffeef2", surface: "#fff5f7", surface2: "#f5d9df", surface3: "#eec6cf", surface4: "#e3b0bd",
            border: "#dba3b0", borderSoft: "#eecfd6", ink: "#000000", inkDim: "#5a3038", inkFaint: "#8f6670",
            accent: "#a8375f"
        },
        photon: {
            canvas: "#ffffff", surface: "#ffffff", surface2: "#f4f6f8", surface3: "#e9edf1", surface4: "#dce1e8",
            border: "#dfe3e8", borderSoft: "#eef1f4", ink: "#1a1a1a", inkDim: "#5a6472", inkFaint: "#8b93a0",
            accent: "#3a8fd9"
        },
        burichan: {
            canvas: "#eef2ff", surface: "#f5f7fc", surface2: "#d6daf0", surface3: "#c3c9e3", surface4: "#b1b9d8",
            border: "#a4accf", borderSoft: "#d3daea", ink: "#000000", inkDim: "#3d4266", inkFaint: "#7a80a3",
            accent: "#3355aa"
        },
    })
    readonly property var chanThemeIds: ["yotsuba", "yotsubab", "futaba", "photon", "burichan"]
    readonly property var chanThemeNames: ({ yotsuba: "Yotsuba", yotsubab: "Yotsuba B", futaba: "Futaba", photon: "Photon", burichan: "Burichan" })
    readonly property bool isChanTheme: chanThemeIds.indexOf(SettingsManager.theme) !== -1
    readonly property var activeChanPalette: isChanTheme ? chanPalettes[SettingsManager.theme] : null

    readonly property bool dark: isChanTheme
        ? false
        : (SettingsManager.theme === "light" ? false : (SettingsManager.theme === "dark" ? true : systemDark))

    readonly property color accent: isChanTheme ? activeChanPalette.accent : SettingsManager.accentColor
    readonly property color accent2: "#8b5cf6"

    readonly property color canvas:     isChanTheme ? activeChanPalette.canvas     : (dark ? "#0a0b11" : "#f5f6fb")
    readonly property color surface:    isChanTheme ? activeChanPalette.surface    : (dark ? "#12141c" : "#ffffff")
    readonly property color surface2:   isChanTheme ? activeChanPalette.surface2   : (dark ? "#171a24" : "#eef0f7")
    readonly property color surface3:   isChanTheme ? activeChanPalette.surface3   : (dark ? "#1e2230" : "#e3e6f0")
    readonly property color surface4:   isChanTheme ? activeChanPalette.surface4   : (dark ? "#262b3c" : "#d7dbe8")
    readonly property color border:     isChanTheme ? activeChanPalette.border     : (dark ? "#2b3145" : "#d5d9e6")
    readonly property color borderSoft: isChanTheme ? activeChanPalette.borderSoft : (dark ? "#1d2130" : "#e8eaf2")
    readonly property color ink:        isChanTheme ? activeChanPalette.ink        : (dark ? "#eef0f6" : "#12141d")
    readonly property color inkDim:     isChanTheme ? activeChanPalette.inkDim     : (dark ? "#a3aac0" : "#4c5268")
    readonly property color inkFaint:   isChanTheme ? activeChanPalette.inkFaint   : (dark ? "#666f88" : "#868da5")

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
    // build accent-tinted washes without repeating Qt.rgba(c.r, c.g,
    // c.b, a) at every call site.
    function withAlpha(c, a) {
        return Qt.rgba(c.r, c.g, c.b, a)
    }
}
