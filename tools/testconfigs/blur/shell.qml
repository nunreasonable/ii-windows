// Manual test for blur behind panels (quickshell branch `blur`, src/windows/blur.cpp).
// Its rules are in defaults/windows/layerrules.json next to this file.
//
//   qs -p tools/testconfigs/blur
//   qs ipc -p tools/testconfigs/blur call blur toggleEnabled
//   qs ipc -p tools/testconfigs/blur call blur toggleCards
//   qs ipc -p tools/testconfigs/blur call blur state
//
// Put a busy window (a web page, an image) behind the cards. Expected:
// - "card" (top left): blurred behind the rounded rectangle only, sharp in the 40px margin
//   around it (where the fake shadow is), corners rounded like the card. Clicks on the margin
//   reach the window below; clicks on the card log PRESS card.
// - "nested": blur only behind the inner rectangle (0.4 over 0.4 shows 0.64 > 0.5), not behind
//   the outer one (0.4 < 0.5).
// - "moving": the blur follows the card while it slides.
// - "opaque": no blur (and no backdrop window: tools/blur-check.ps1 lists one window less).
// - "whole": the whole window blurred (ignoreAlpha null), square corners.
// - "off": never blurred.
// - "overlay" (Overlay layer, overlaps "card"): its blur shows "card" blurred underneath, and
//   card's blur never covers overlay's contents.
// - toggleCards hides and shows every card: no blur left behind while hidden.
// With QS_WINDOWS_BLUR_DEBUG=1 every blurred shape gets a red tint.

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

		Rectangle { // fake shadow in the margin: must stay sharp
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
