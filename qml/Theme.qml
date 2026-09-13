pragma Singleton
import QtQuick
import Channex

// Approximates the CSS custom-property theme in src/index.css (dark
// canvas/surface/ink scale, light-mode overrides, user-selectable
// accent). Exact hex values are a placeholder pass, not pixel-matched
// against the Tailwind config - fine-tune once both apps are meant to
// look identical.
QtObject {
    readonly property bool systemDark: Qt.styleHints.colorScheme === Qt.Dark

    readonly property bool dark: SettingsManager.theme === "light"
        ? false
        : (SettingsManager.theme === "dark" ? true : systemDark)

    readonly property color accent: SettingsManager.accentColor
    readonly property color accent2: "#8b5cf6"

    readonly property color canvas:     dark ? "#0c0e11" : "#f7f8fa"
    readonly property color surface:    dark ? "#14171c" : "#ffffff"
    readonly property color surface2:   dark ? "#1a1e24" : "#f0f1f3"
    readonly property color surface3:   dark ? "#20252c" : "#e6e8eb"
    readonly property color surface4:   dark ? "#262c34" : "#dcdee2"
    readonly property color border:     dark ? "#2a2f37" : "#d8dade"
    readonly property color borderSoft: dark ? "#1f242b" : "#e8e9ec"
    readonly property color ink:        dark ? "#e8eaed" : "#16181c"
    readonly property color inkDim:     dark ? "#a8adb5" : "#52565e"
    readonly property color inkFaint:   dark ? "#6b7280" : "#8b8f97"

    readonly property color danger: "#ef4444"
    readonly property color success: "#22c55e"

    readonly property int radiusSm: 6
    readonly property int radiusMd: 10
    readonly property int radiusLg: 16

    readonly property var accentPresets: [
        { name: "Mint",   value: "#6ee7c9" },
        { name: "Violet", value: "#8b5cf6" },
        { name: "Sky",    value: "#38bdf8" },
        { name: "Rose",   value: "#fb7185" },
        { name: "Amber",  value: "#fbbf24" },
        { name: "Lime",   value: "#a3e635" },
    ]
}
