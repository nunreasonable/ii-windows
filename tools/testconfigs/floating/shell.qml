import QtQuick
import Quickshell
import Quickshell.Io

ShellRoot {
	FloatingWindow {
		id: win
		implicitWidth: 420
		implicitHeight: 140
		color: "#1d1b20"
		Text {
			anchors.centerIn: parent
			color: "#e6e0e9"
			font.pixelSize: 18
			text: "quickshell core on Windows\n" + Quickshell.shellDir + "\npings: " + ipc.pings
		}
	}

	IpcHandler {
		id: ipc
		target: "test"
		property int pings: 0
		function ping(): string { pings++; return "pong " + pings; }
	}
}
