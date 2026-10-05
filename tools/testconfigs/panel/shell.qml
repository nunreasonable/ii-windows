import QtQuick
import QtQuick.Controls
import Quickshell
import Quickshell.Io
import Quickshell.Wayland

ShellRoot {
	id: root

	property bool barVisible: true

	IpcHandler {
		target: "test"

		function toggleBar(): void {
			root.barVisible = !root.barVisible;
		}

		function ping(): string {
			return "pong";
		}
	}

	PanelWindow {
		id: barWindow

		visible: root.barVisible
		color: "transparent"
		implicitHeight: 40
		exclusiveZone: 40

		anchors {
			left: true
			top: true
			right: true
		}

		WlrLayershell.layer: WlrLayer.Top
		WlrLayershell.namespace: "quickshell:bar"

		mask: Region { item: bar }

		Rectangle {
			id: bar

			anchors.fill: parent
			anchors.margins: 4
			anchors.leftMargin: 120
			anchors.rightMargin: 120
			radius: 12
			color: Qt.rgba(0.1, 0.1, 0.15, 0.75)

			Row {
				anchors.centerIn: parent
				spacing: 16

				Text {
					text: Qt.formatDateTime(clock.date, "hh:mm:ss")
					color: "white"
					font.pixelSize: 16
				}

				Text {
					text: barWindow.screen ? barWindow.screen.name : "no screen"
					color: "white"
					font.pixelSize: 16
				}

				Text {
					text: "bar " + barWindow.width + "x" + barWindow.height + " dpr " + barWindow.devicePixelRatio
					color: "white"
					font.pixelSize: 16
				}
			}

			MouseArea {
				anchors.fill: parent
				onClicked: console.log("bar clicked at", mouse.x, mouse.y)
			}
		}

		SystemClock {
			id: clock
			precision: SystemClock.Seconds
		}
	}

	PanelWindow {
		color: "transparent"
		implicitWidth: 320
		implicitHeight: 120

		anchors {
			right: true
			bottom: true
		}

		margins {
			right: 16
			bottom: 16
		}

		WlrLayershell.layer: WlrLayer.Overlay
		WlrLayershell.keyboardFocus: WlrKeyboardFocus.OnDemand
		WlrLayershell.namespace: "quickshell:overlay"

		Rectangle {
			anchors.fill: parent
			radius: 12
			color: Qt.rgba(0.15, 0.1, 0.1, 0.85)

			Column {
				anchors.centerIn: parent
				spacing: 8

				Text {
					text: "Overlay, focus OnDemand"
					color: "white"
				}

				TextField {
					width: 260
					placeholderText: "type here"
				}
			}
		}
	}

	PanelWindow {
		exclusionMode: ExclusionMode.Ignore

		anchors {
			left: true
			top: true
			right: true
			bottom: true
		}

		WlrLayershell.layer: WlrLayer.Background
		WlrLayershell.namespace: "quickshell:background"

		Rectangle {
			anchors.fill: parent

			gradient: Gradient {
				GradientStop { position: 0.0; color: "#1b2838" }
				GradientStop { position: 1.0; color: "#4b2e5a" }
			}

			Text {
				anchors.centerIn: parent
				text: "Background layer"
				color: "#80ffffff"
				font.pixelSize: 48
			}
		}
	}

	FloatingWindow {
		title: "Quickshell floating window"
		implicitWidth: 400
		implicitHeight: 200
		color: "#202020"

		Column {
			anchors.centerIn: parent
			spacing: 8

			Text {
				text: "FloatingWindow"
				color: "white"
			}

			Button {
				text: "Toggle bar"
				onClicked: root.barVisible = !root.barVisible
			}
		}
	}
}
