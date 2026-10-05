import QtQuick
import Quickshell
import Quickshell.Wayland

ShellRoot {
	component Probe: Rectangle {
		required property string label
		anchors.fill: parent
		color: "#80304050"
		Text { anchors.centerIn: parent; text: parent.label; color: "white" }
		MouseArea {
			anchors.fill: parent
			hoverEnabled: true
			onPressed: console.log("PRESS", parent.label)
			onEntered: console.log("ENTER", parent.label)
		}
	}

	PanelWindow {
		anchors { left: true; top: true }
		margins { left: 100; top: 200 }
		implicitWidth: 200; implicitHeight: 100
		exclusionMode: ExclusionMode.Ignore
		color: "transparent"
		Probe { label: "panel-none" }
	}

	PanelWindow {
		anchors { left: true; top: true }
		margins { left: 400; top: 200 }
		implicitWidth: 200; implicitHeight: 100
		exclusionMode: ExclusionMode.Ignore
		WlrLayershell.keyboardFocus: WlrKeyboardFocus.OnDemand
		color: "#202020"
		Probe { label: "panel-ondemand" }
	}

	PanelWindow {
		anchors { left: true; top: true }
		margins { left: 700; top: 200 }
		implicitWidth: 200; implicitHeight: 100
		exclusionMode: ExclusionMode.Ignore
		WlrLayershell.layer: WlrLayer.Bottom
		color: "#202020"
		Probe { label: "panel-bottom" }
	}

	FloatingWindow {
		implicitWidth: 200; implicitHeight: 100
		color: "#202020"
		Probe { label: "floating" }
	}
}
