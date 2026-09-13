import QtQuick
import Channex

// Loads a monochrome SVG from resources/icons/ui/<name>.svg and tints it
// to `color` by rewriting the SVG's own "#000000" to the target color
// and re-encoding it as a data: URL, rather than recoloring at render
// time with a shader effect - MultiEffect's colorization rendered icons
// as flat black regardless of colorizationColor on this Qt/RHI setup,
// so this sidesteps that entirely and is guaranteed correct since it
// edits the actual pixel data the Image loads. The SVG text itself
// comes from IconProvider (C++) rather than QML's XMLHttpRequest, which
// disables local-file/qrc reads by default.
Item {
    id: root
    property string name: "grid"
    property color color: Theme.ink
    property real iconSize: 16

    implicitWidth: iconSize
    implicitHeight: iconSize

    Image {
        anchors.fill: parent
        source: root.coloredSource()
        sourceSize.width: root.iconSize * 2
        sourceSize.height: root.iconSize * 2
        fillMode: Image.PreserveAspectFit
        smooth: true
        asynchronous: false
    }

    function coloredSource() {
        var svg = IconProvider.rawSvg(root.name)
        if (!svg || svg.length === 0) return ""
        var hex = "#" + root.color.toString().replace("#", "").slice(-6)
        var colored = svg.split("#000000").join(hex)
        return "data:image/svg+xml;utf8," + encodeURIComponent(colored)
    }
}
