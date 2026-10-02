// Windowless test for the native Quickshell.Services.Notifications on Windows (no windows are
// created, safe to run on a shared desktop):
//
//   qs -p tools/testconfigs/notifications
//
// Sends notifications through notifySend (the notify-send stand-in ii's sendDesktop uses),
// prints what arrives, invokes actions, replaces/expires/dismisses, prints the Windows toast
// mirror's state and every mirrored toast. Mirrored toasts whose summary starts with
// "ii-windows probe" are dismissed after 1.5 s, which removes them from the Windows notification
// center. Show one from PowerShell while this runs to exercise the mirror. Quits after
// QS_NOTIF_TEST_SECONDS (default 20); QS_NOTIF_TEST_PNG adds a notification whose app icon is
// that PNG file. tools/notif-test.ps1 runs all of this on the VM.

import QtQuick
import Quickshell
import Quickshell.Services.Notifications
import Quickshell.Windows as W

ShellRoot {
	id: root

	property var ids: ({})
	property int step: 0

	function log(...args) {
		console.info("[test]", ...args);
	}

	function describe(n) {
		return JSON.stringify({
			id: n.id,
			appName: n.appName,
			appIcon: n.appIcon,
			summary: n.summary,
			body: n.body,
			urgency: NotificationUrgency.toString(n.urgency),
			urgencyString: n.urgency.toString(),
			image: n.image,
			desktopEntry: n.desktopEntry,
			expireTimeout: n.expireTimeout,
			transient: n.transient,
			resident: n.resident,
			lastGeneration: n.lastGeneration,
			hints: n.hints,
			actions: n.actions.map(a => a.identifier + "=" + a.text),
		});
	}

	function tracked(id) {
		return server.trackedNotifications.values.find(n => n.id === id) ?? null;
	}

	function invoke(id, identifier) {
		const n = tracked(id);
		if (!n) return log("invoke: no tracked notification", id);
		const action = n.actions.find(a => a.identifier === identifier);
		if (!action) return log("invoke: no action", identifier, "on", id);
		log("invoking", identifier, "on", id);
		action.invoke();
		log("after invoke: tracked", id, "=", tracked(id) !== null);
	}

	NotificationServer {
		id: server
		keepOnReload: false
		actionsSupported: true
		bodyMarkupSupported: true
		bodySupported: true
		imageSupported: true
		persistenceSupported: true

		onNotification: n => {
			n.tracked = true;
			const mirrored = n.hints["x-windows-toast-id"] !== undefined;
			root.log(mirrored ? "mirrored" : "notification", root.describe(n));
			if (n.appIcon !== "") {
				// Loads through the icon provider like ii's NotificationAppIcon; a failure shows
				// up as "Could not load icon" in the log.
				const icon = iconCheck.createObject(root, {
					label: "notification " + n.id,
					sourceSize: Qt.size(64, 64),
					source: Quickshell.iconPath(n.appIcon, "image-missing"),
				});
				root.log("icon", icon.label, ["null", "ready", "loading", "error"][icon.status], icon.implicitWidth + "x" + icon.implicitHeight, icon.source);
			}
			const id = n.id;
			n.closed.connect(reason => root.log("closed", id, NotificationCloseReason.toString(reason)));

			if (mirrored && n.summary.startsWith("ii-windows probe")) {
				dismissLater.createObject(root, { target: n });
			}
		}

		onActionInvoked: (id, action) => root.log("actionInvoked", id, action)
		onSystemNotificationAccessChanged: root.log("access", SystemNotificationAccess.toString(systemNotificationAccess))
		onSystemMirrorActiveChanged: root.log("mirror active", systemMirrorActive)
	}

	Component {
		id: iconCheck
		Image {
			property string label
		}
	}

	Component {
		id: dismissLater
		Timer {
			property var target
			interval: 1500
			running: true
			onTriggered: {
				root.log("dismissing mirrored", target.id);
				target.dismiss();
				destroy();
			}
		}
	}

	Timer {
		interval: 500
		repeat: true
		running: true
		onTriggered: {
			switch (root.step++) {
			case 0:
				root.log("start: access", SystemNotificationAccess.toString(server.systemNotificationAccess),
					"mirror", server.mirrorSystemNotifications, server.systemMirrorActive,
					"tracked", server.trackedNotifications.values.length,
					"NotificationSettings.openSettings", typeof W.NotificationSettings.openSettings,
					"openAccessSettings", typeof W.NotificationSettings.openAccessSettings);
				break;
			case 1:
				root.ids.battery = server.notifySend("Low battery", "Consider plugging in your device",
					["-u", "critical", "-a", "Shell", "--hint=int:transient:1"]);
				root.log("notifySend returned", root.ids.battery, "tracked yet:", root.tracked(root.ids.battery) !== null);
				break;
			case 2:
				root.ids.actions = server.notifySend("Actions", "Body with <b>markup</b>", [
					"-a", "Shell", "-A", "ok=OK", "-A", "Later", "-i", "C:/Windows/System32/notepad.exe",
					"-t", "5000", "-h", "string:desktop-entry:notepad", "-c", "im.received",
					"--hint=string:image-path:C:/Windows/Web/Wallpaper/Windows/img0.jpg"]);
				root.ids.plain = server.notifySend("Plain", "no args");
				// A PNG app icon, the way mirrored toasts hand over cached app logos.
				if (Quickshell.env("QS_NOTIF_TEST_PNG")) {
					server.notifySend("PNG icon", "app icon from a PNG file", ["-a", "Shell", "-i", Quickshell.env("QS_NOTIF_TEST_PNG")]);
				}
				break;
			case 3:
				root.ids.replaced = server.notifySend("Replaced title", "replaced body",
					["-r", String(root.ids.actions), "-a", "Shell", "-A", "ok=OK", "-ulow"]);
				root.log("replace returned", root.ids.replaced);
				break;
			case 4:
				root.log("after replace", root.describe(root.tracked(root.ids.actions)));
				root.invoke(root.ids.actions, "ok");
				break;
			case 5:
				root.ids.resident = server.notifySend("Resident", "stays after an action",
					["-h", "boolean:resident:true", "-A", "x=X", "-a", "Shell", "-e"]);
				break;
			case 6:
				root.invoke(root.ids.resident, "x");
				root.tracked(root.ids.resident)?.dismiss();
				root.tracked(root.ids.battery)?.expire();
				root.log("tracked now", server.trackedNotifications.values.map(n => n.id + ":" + n.summary));
				break;
			default:
				const limit = Number(Quickshell.env("QS_NOTIF_TEST_SECONDS") ?? 20) * 2;
				if (root.step >= limit) {
					root.log("end: access", SystemNotificationAccess.toString(server.systemNotificationAccess),
						"mirror active", server.systemMirrorActive,
						"tracked", server.trackedNotifications.values.map(n => n.id + ":" + n.summary));
					Qt.quit();
				}
			}
		}
	}
}
