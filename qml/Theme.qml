pragma Singleton
import QtQuick
import Channex

// Material Design 3 token set - every component reads color, shape,
// type, motion, and state-layer values from here.
//
// Color roles (primary / secondary / tertiary / error + containers,
// the five surface-container tiers, outline / outlineVariant, ...) are
// generated from a single seed color the same way M3's dynamic color
// does it: each role is a fixed *tone* (perceptual lightness, CIE L*)
// of a tonal palette that shares the seed's hue. Real M3 uses the HCT
// color space for this; here each tone is found by binary-searching
// HSL lightness until the color's L* hits the target tone, which keeps
// the important property - every role has the right contrast against
// its "on" color regardless of which hue the seed is.
//
// SettingsManager.theme selects "dark" / "light" / "system" (seeded
// by the user's accentColor) or one of five classic imageboard skins -
// "yotsuba" / "yotsubab" / "futaba" / "photon" / "burichan" - modeled
// after 4chan's own theme-selector palettes. A skin keeps its own
// authentic surface colors and uses its traditional link color as the
// primary, with the remaining roles derived from that.
//
// The pre-M3 token names (canvas, surface2, ink, inkDim, accent, ...)
// remain as aliases onto the closest M3 role so nothing that still
// reads them breaks.
QtObject {
    id: theme

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

    // ------------------------------------------------------------------
    // Tonal palette generation
    // ------------------------------------------------------------------

    function _lin(c) { return c <= 0.04045 ? c / 12.92 : Math.pow((c + 0.055) / 1.055, 2.4) }

    // CIE L* (0-100) of a color - M3's "tone".
    function lstar(c) {
        var y = 0.2126 * _lin(c.r) + 0.7152 * _lin(c.g) + 0.0722 * _lin(c.b)
        return y > 216 / 24389 ? 116 * Math.cbrt(y) - 16 : (24389 / 27) * y
    }

    // A color with hue `h` and HSL saturation `s` whose L* is `t`.
    function tone(h, s, t) {
        if (t <= 0) return Qt.rgba(0, 0, 0, 1)
        if (t >= 100) return Qt.rgba(1, 1, 1, 1)
        var lo = 0, hi = 1, c = Qt.hsla(h, s, 0.5, 1)
        for (var i = 0; i < 18; i++) {
            var mid = (lo + hi) / 2
            c = Qt.hsla(h, s, mid, 1)
            if (lstar(c) < t) lo = mid; else hi = mid
        }
        return c
    }

    readonly property color seed: isChanTheme ? activeChanPalette.accent : SettingsManager.accentColor
    readonly property real seedHue: seed.hslHue < 0 ? 0.62 : seed.hslHue
    readonly property real seedSat: Math.max(0.35, seed.hslSaturation)

    // M3 key palettes: primary at the seed's own chroma, secondary a
    // muted version of it, tertiary rotated ~60deg, neutrals barely
    // tinted with the seed hue.
    readonly property var pal: ({
        p:  { h: seedHue, s: Math.min(0.85, seedSat) },
        s:  { h: seedHue, s: 0.22 },
        t:  { h: (seedHue + 0.166) % 1, s: Math.min(0.6, seedSat * 0.8) },
        n:  { h: seedHue, s: 0.08 },
        nv: { h: seedHue, s: 0.16 },
        e:  { h: 0.99, s: 0.75 },
    })
    function _t(key, t) { return tone(pal[key].h, pal[key].s, t) }
    function _d(darkTone, lightTone) { return dark ? darkTone : lightTone }

    // ------------------------------------------------------------------
    // Color roles
    // ------------------------------------------------------------------
    //
    // M3's "on" roles carry a Color suffix (onSurfaceColor, not
    // onSurface): QML parses `onSurface: ...` as a handler for a
    // signal named after the existing `surface` property, so a
    // property declared with that name silently never holds a color.

    readonly property color primary:              isChanTheme ? activeChanPalette.accent : _t("p", _d(80, 40))
    readonly property color onPrimaryColor:            isChanTheme ? "#ffffff" : _t("p", _d(20, 100))
    readonly property color primaryContainer:     _t("p", _d(30, 90))
    readonly property color onPrimaryContainerColor:   _t("p", _d(90, 10))
    readonly property color inversePrimary:       _t("p", _d(40, 80))

    readonly property color secondary:            _t("s", _d(80, 40))
    readonly property color onSecondaryColor:          _t("s", _d(20, 100))
    readonly property color secondaryContainer:   isChanTheme ? activeChanPalette.surface3 : _t("s", _d(30, 90))
    readonly property color onSecondaryContainerColor: isChanTheme ? activeChanPalette.ink : _t("s", _d(90, 10))

    readonly property color tertiary:             _t("t", _d(80, 40))
    readonly property color onTertiaryColor:           _t("t", _d(20, 100))
    readonly property color tertiaryContainer:    _t("t", _d(30, 90))
    readonly property color onTertiaryContainerColor:  _t("t", _d(90, 10))

    readonly property color error:                _t("e", _d(80, 40))
    readonly property color onErrorColor:              _t("e", _d(20, 100))
    readonly property color errorContainer:       _t("e", _d(30, 90))
    readonly property color onErrorContainerColor:     _t("e", _d(90, 10))

    readonly property color surface:                isChanTheme ? activeChanPalette.canvas   : _t("n", _d(6, 98))
    readonly property color surfaceDim:             isChanTheme ? activeChanPalette.surface2 : _t("n", _d(6, 87))
    readonly property color surfaceBright:          isChanTheme ? activeChanPalette.surface  : _t("n", _d(24, 98))
    readonly property color surfaceContainerLowest: isChanTheme ? activeChanPalette.surface  : _t("n", _d(4, 100))
    // A skin's own "surface" is nearly its canvas color, which would make
    // low-tier containers vanish; sit them halfway to the post color.
    readonly property color surfaceContainerLow:    isChanTheme ? Qt.tint(activeChanPalette.canvas, Qt.rgba(Qt.color(activeChanPalette.surface2).r, Qt.color(activeChanPalette.surface2).g, Qt.color(activeChanPalette.surface2).b, 0.5)) : _t("n", _d(10, 96))
    readonly property color surfaceContainer:       isChanTheme ? activeChanPalette.surface2 : _t("n", _d(12, 94))
    readonly property color surfaceContainerHigh:   isChanTheme ? activeChanPalette.surface3 : _t("n", _d(17, 92))
    readonly property color surfaceContainerHighest:isChanTheme ? activeChanPalette.surface4 : _t("n", _d(22, 90))
    readonly property color onSurfaceColor:              isChanTheme ? activeChanPalette.ink      : _t("n", _d(90, 10))
    readonly property color onSurfaceVariantColor:       isChanTheme ? activeChanPalette.inkDim   : _t("nv", _d(80, 30))
    readonly property color outline:                isChanTheme ? activeChanPalette.inkFaint : _t("nv", _d(60, 50))
    readonly property color outlineVariant:         isChanTheme ? activeChanPalette.border   : _t("nv", _d(30, 80))
    readonly property color inverseSurface:         _t("n", _d(90, 20))
    readonly property color inverseOnSurface:       _t("n", _d(20, 95))
    readonly property color scrim: "#000000"
    readonly property color shadow: "#000000"

    // Not M3 roles, but this app needs them: download/success state,
    // imageboard >greentext, and >>quote links in rendered comments.
    readonly property color success:   dark ? "#7dd99a" : "#1e6b3a"
    readonly property color greentext: dark ? "#a5d46a" : "#4b7a1c"
    readonly property color quoteLink: dark ? tertiary : (isChanTheme ? "#d00000" : tertiary)

    // ------------------------------------------------------------------
    // Legacy aliases (pre-M3 token names)
    // ------------------------------------------------------------------

    readonly property color accent: primary
    readonly property color accent2: tertiary
    readonly property color canvas: surface
    readonly property color surface2: surfaceContainer
    readonly property color surface3: surfaceContainerHigh
    readonly property color surface4: surfaceContainerHighest
    readonly property color border: outlineVariant
    readonly property color borderSoft: outlineVariant
    readonly property color ink: onSurfaceColor
    readonly property color inkDim: onSurfaceVariantColor
    readonly property color inkFaint: outline
    readonly property color danger: error
    readonly property color shadowColor: shadow

    // ------------------------------------------------------------------
    // Shape scale
    // ------------------------------------------------------------------

    readonly property int shapeExtraSmall: 4
    readonly property int shapeSmall: 8
    readonly property int shapeMedium: 12
    readonly property int shapeLarge: 16
    readonly property int shapeLargeIncreased: 20
    readonly property int shapeExtraLarge: 28
    readonly property int shapeFull: 9999

    readonly property int radiusSm: shapeSmall
    readonly property int radiusMd: shapeMedium
    readonly property int radiusLg: shapeLarge
    readonly property int radiusXl: shapeExtraLarge

    readonly property int spacingXs: 4
    readonly property int spacingSm: 8
    readonly property int spacingMd: 12
    readonly property int spacingLg: 16
    readonly property int spacingXl: 24

    // ------------------------------------------------------------------
    // Type scale (sizes in px; weights via Font.*)
    // ------------------------------------------------------------------

    readonly property int displaySmall: 36
    readonly property int headlineMedium: 28
    readonly property int headlineSmall: 24
    readonly property int titleLarge: 22
    readonly property int titleMedium: 16
    readonly property int titleSmall: 14
    readonly property int bodyLarge: 16
    readonly property int bodyMedium: 14
    readonly property int bodySmall: 12
    readonly property int labelLarge: 14
    readonly property int labelMedium: 12
    readonly property int labelSmall: 11

    // ------------------------------------------------------------------
    // State layers + motion
    // ------------------------------------------------------------------

    readonly property real hoverOpacity: 0.08
    readonly property real focusOpacity: 0.10
    readonly property real pressedOpacity: 0.10
    readonly property real draggedOpacity: 0.16
    readonly property real disabledContentOpacity: 0.38
    readonly property real disabledContainerOpacity: 0.12

    // Pass as `easing.bezierCurve` with `easing.type: Easing.BezierSpline`.
    readonly property var easeStandard: [0.2, 0.0, 0.0, 1.0, 1.0, 1.0]
    readonly property var easeEmphasizedDecel: [0.05, 0.7, 0.1, 1.0, 1.0, 1.0]
    readonly property var easeEmphasizedAccel: [0.3, 0.0, 0.8, 0.15, 1.0, 1.0]

    readonly property int durShort: 150
    readonly property int durMedium: 250
    readonly property int durLong: 400

    readonly property var accentPresets: [
        { name: "Baseline", value: "#6750a4" },
        { name: "Teal",     value: "#006a6a" },
        { name: "Blue",     value: "#1f6feb" },
        { name: "Green",    value: "#386a20" },
        { name: "Rose",     value: "#b4245f" },
        { name: "Amber",    value: "#9a6700" },
        { name: "Mint",     value: "#6ee7c9" },
        { name: "Violet",   value: "#8b5cf6" },
    ]

    // Shorthand for a translucent variant of any color.
    function withAlpha(c, a) {
        return Qt.rgba(c.r, c.g, c.b, a)
    }

    // A color with a state layer of `layer` at `alpha` composited on top
    // - for places that need the result as one solid color rather than
    // an overlay item (e.g. a tonal surface that also needs a border).
    function blend(base, layer, alpha) {
        return Qt.tint(base, Qt.rgba(layer.r, layer.g, layer.b, alpha))
    }

    function hex(c) {
        return "#" + c.toString().replace("#", "").slice(-6)
    }

    // Comment HTML -> one-line plain text for card excerpts: line breaks
    // become spaces, tags are dropped, and entities (&gt; &#039; ...)
    // decoded rather than shown literally.
    function plainText(html) {
        if (!html) return ""
        return html.replace(/<br\s*\/?>/gi, " ")
                   .replace(/<[^>]*>/g, "")
                   .replace(/&#(\d+);/g, function (m, n) { return String.fromCharCode(parseInt(n, 10)) })
                   .replace(/&#x([0-9a-f]+);/gi, function (m, n) { return String.fromCharCode(parseInt(n, 16)) })
                   .replace(/&gt;/g, ">").replace(/&lt;/g, "<").replace(/&quot;/g, "\"")
                   .replace(/&nbsp;/g, " ").replace(/&amp;/g, "&")
    }

    // CommentFormatter (C++) emits fixed inline colors for quote links,
    // greentext, and spoilers; re-point them at the current scheme's
    // roles so rendered comments follow the theme.
    function themeComment(html) {
        if (!html) return ""
        return html.split("#7fb0e0").join(hex(quoteLink))
                   .split("#86b661").join(hex(greentext))
                   .split("#8a8f98").join(hex(outline))
                   .split("#1a1a1a").join(hex(onSurfaceColor))
    }
}
