import QtQuick
import Quickshell

FloatingWindow {
    id: win
    implicitWidth: 900
    implicitHeight: 600
    color: "white"
    property string fam: Quickshell.env("IIW_FONTTEST_FAMILY") || "Bahnschrift"
    property int count: parseInt(Quickshell.env("IIW_FONTTEST_COUNT") || "800")
    property int shown: 0
    Flow {
        anchors.fill: parent
        Repeater {
            model: win.shown
            Text {
                required property int index
                text: "Ag" + index
                font.family: win.fam
                font.pixelSize: 14
                font.variableAxes: ({ "wght": 300 + index * 0.37 })
            }
        }
    }
    Timer {
        interval: 50
        repeat: true
        running: win.shown < win.count
        onTriggered: {
            win.shown = Math.min(win.count, win.shown + 25);
            if (win.shown % 100 === 0) console.warn("FONTTEST shown=" + win.shown);
        }
    }
}
