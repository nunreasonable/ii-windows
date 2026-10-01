import QtQuick
import QtQuick.Window

Window {
	width: 520; height: 180
	visible: true
	color: "transparent"
	flags: Qt.FramelessWindowHint | Qt.WindowStaysOnTopHint
	Rectangle {
		anchors.fill: parent
		radius: 24
		color: "#cc1d1b20"
		border.color: "#d0bcff"; border.width: 2
		Column {
			anchors.centerIn: parent
			spacing: 8
			Text { color: "#e6e0e9"; font.pixelSize: 22; text: "ii-windows toolchain ok" }
			Text { color: "#cac4d0"; font.pixelSize: 14; text: winrtStatus }
			Text { color: "#cac4d0"; font.pixelSize: 14; text: "graphicsApi: " + GraphicsInfo.api }
		}
	}
}
