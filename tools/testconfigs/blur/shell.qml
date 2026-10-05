import QtQuick
import Quickshell
import Quickshell.Io
import Quickshell.Wayland
import Quickshell.Windows

ShellRoot {
	id: root

	property bool cardsVisible: true

	IpcHandler {
		target: "blur"

		function toggleEnabled(): void {
			BackdropBlur.enabled = !BackdropBlur.enabled;
		}

		function toggleCards(): void {
			root.cardsVisible = !root.cardsVisible;
		}

		function state(): string {
			return `enabled ${BackdropBlur.enabled} available ${BackdropBlur.available} rules ${BackdropBlur.rules.length} from ${BackdropBlur.configPath}`;
		}
	}

	component Card: Rectangle {
		property string label
		radius: 24
		border.width: 1
		border.color: Qt.rgba(1, 1, 1, 0.3)
		Text {
			anchors.centerIn: parent
			text: parent.label
			color: "white"
			font.pixelSize: 20
		}
		MouseArea {
			anchors.fill: parent
			onPressed: console.log("PRESS", parent.label)
		}
	}

	component TestPanel: PanelWindow {
		visible: root.cardsVisible
		color: "transparent"
		exclusionMode: ExclusionMode.Ignore
		anchors { left: true; top: true }
		implicitWidth: 360
		implicitHeight: 240
	}

	TestPanel {
		margins { left: 80; top: 120 }
		WlrLayershell.namespace: "blurtest"
		mask: Region { item: card }

		Rectangle {
			anchors.fill: card
			anchors.margins: -30
			radius: 50
			color: Qt.rgba(0, 0, 0, 0.25)
		}
		Card {
			id: card
			anchors.fill: parent
			anchors.margins: 40
			color: Qt.rgba(0.1, 0.1, 0.15, 0.6)
			label: "card"
		}
	}

	TestPanel {
		margins { left: 480; top: 120 }
		WlrLayershell.namespace: "blurtest-nested"
		Card {
			anchors.fill: parent
			anchors.margins: 20
			color: Qt.rgba(0.1, 0.1, 0.15, 0.4)
			label: ""
			Card {
				anchors.centerIn: parent
				width: 160; height: 100
				color: Qt.rgba(0.1, 0.1, 0.15, 0.4)
				label: "nested"
			}
		}
	}

	TestPanel {
		margins { left: 80; top: 400 }
		implicitWidth: 760
		WlrLayershell.namespace: "blurtest-moving"
		mask: Region { item: moving }
		Card {
			id: moving
			width: 200; height: 160; y: 40
			color: Qt.rgba(0.15, 0.1, 0.1, 0.6)
			label: "moving"
			SequentialAnimation on x {
				loops: Animation.Infinite
				NumberAnimation { from: 0; to: 560; duration: 3000; easing.type: Easing.InOutQuad }
				NumberAnimation { from: 560; to: 0; duration: 3000; easing.type: Easing.InOutQuad }
			}
		}
	}

	TestPanel {
		margins { left: 880; top: 120 }
		WlrLayershell.namespace: "blurtest-opaque"
		Card { anchors.fill: parent; anchors.margins: 20; color: "#303040"; label: "opaque" }
	}

	TestPanel {
		margins { left: 880; top: 400 }
		color: Qt.rgba(0.1, 0.15, 0.1, 0.3)
		WlrLayershell.namespace: "blurtest-whole"
		Text { anchors.centerIn: parent; text: "whole"; color: "white"; font.pixelSize: 20 }
	}

	TestPanel {
		margins { left: 1280; top: 120 }
		WlrLayershell.namespace: "blurtest-off"
		Card { anchors.fill: parent; anchors.margins: 20; color: Qt.rgba(0.1, 0.1, 0.15, 0.6); label: "off" }
	}

	TestPanel {
		margins { left: 260; top: 220 }
		WlrLayershell.layer: WlrLayer.Overlay
		WlrLayershell.namespace: "blurtest-overlay"
		mask: Region { item: overlayCard }
		Card {
			id: overlayCard
			anchors.fill: parent
			anchors.margins: 40
			color: Qt.rgba(0.1, 0.15, 0.2, 0.6)
			label: "overlay"
		}
	}
}
