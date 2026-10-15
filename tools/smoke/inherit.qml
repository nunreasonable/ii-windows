import QtQuick
import Quickshell

FloatingWindow {
    implicitWidth: 400
    implicitHeight: 120
    color: "white"
    FontLoader {
        id: testFont
        source: "file:///C:/ii-windows/smoketest/inherit/fonts/test.ttf"
    }
    Text {
        anchors.centerIn: parent
        text: "inherit test " + testFont.status
        font.family: testFont.name
        font.pixelSize: 24
    }
    Timer {
        interval: 2500
        running: true
        onTriggered: {
            const cmd = Quickshell.env("IIW_INHERIT_CMD");
            Quickshell.execDetached(cmd ? JSON.parse(cmd) : ["ping", "-n", "40", "127.0.0.1"]);
            console.warn("INHERIT launched");
        }
    }
}
