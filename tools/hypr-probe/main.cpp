#include <cstdio>

#include <qguiapplication.h>
#include <qloggingcategory.h>
#include <qobject.h>
#include <qscreen.h>
#include <qtimer.h>

#include "virtual_desktops.hpp"
#include "window_tracker.hpp"

using namespace qs::windows;

namespace {

QString regGuid(const wchar_t* subkey, const wchar_t* value) {
	GUID guid {};
	DWORD size = sizeof(guid);
	auto rc = RegGetValueW(HKEY_CURRENT_USER, subkey, value, RRF_RT_REG_BINARY, nullptr, &guid, &size);
	if (rc != ERROR_SUCCESS) return QString("(error %1)").arg(rc);
	return VirtualDesktops::guidToString(guid);
}

void dumpRegistry() {
	DWORD session = 0;
	ProcessIdToSessionId(GetCurrentProcessId(), &session);
	auto sessionKey = QString("Software\\Microsoft\\Windows\\CurrentVersion\\Explorer\\SessionInfo\\%1\\VirtualDesktops").arg(session).toStdWString();
	const wchar_t* globalKey = L"Software\\Microsoft\\Windows\\CurrentVersion\\Explorer\\VirtualDesktops";

	DWORD size = 0;
	auto rc = RegGetValueW(HKEY_CURRENT_USER, globalKey, L"VirtualDesktopIDs", RRF_RT_REG_BINARY, nullptr, nullptr, &size);
	printf(
	    "  registry: VirtualDesktopIDs=%s (%lu bytes) global CurrentVirtualDesktop=%s session(%lu) CurrentVirtualDesktop=%s\n",
	    rc == ERROR_SUCCESS ? "present" : "absent",
	    static_cast<unsigned long>(rc == ERROR_SUCCESS ? size : 0),
	    qPrintable(regGuid(globalKey, L"CurrentVirtualDesktop")),
	    static_cast<unsigned long>(session),
	    qPrintable(regGuid(sessionKey.c_str(), L"CurrentVirtualDesktop"))
	);
}

void dumpWindow(const char* prefix, TrackedWindow* w) {
	auto rect = w->rect();
	printf(
	    "%s 0x%llx pid=%u desk=%d act=%d min=%d max=%d fs=%d uwp=%d rect=%d,%d %dx%d screen=%s app=\"%s\" title=\"%s\" exe=%s\n",
	    prefix,
	    static_cast<unsigned long long>(w->address()),
	    w->pid(),
	    w->desktop(),
	    w->activated(),
	    w->minimized(),
	    w->maximized(),
	    w->fullscreen(),
	    w->isUwpFrame(),
	    rect.x(),
	    rect.y(),
	    rect.width(),
	    rect.height(),
	    w->screen() ? qPrintable(w->screen()->name()) : "-",
	    qPrintable(w->appId()),
	    qPrintable(w->title()),
	    qPrintable(w->exePath())
	);
}

void dump(WindowTracker* tracker) {
	auto* vd = tracker->desktops();
	printf(
	    "desktops: %lld current: %lld accessor: %d\n",
	    static_cast<long long>(vd->count()),
	    static_cast<long long>(vd->currentIndex()),
	    vd->accessorLoaded()
	);

	auto i = 0;
	for (const auto& d: vd->desktops()) {
		printf("  [%d] %s \"%s\"\n", i++, qPrintable(VirtualDesktops::guidToString(d.id)), qPrintable(d.name));
	}
	dumpRegistry();

	printf("screens:\n");
	for (auto* screen: QGuiApplication::screens()) {
		auto g = screen->geometry();
		printf("  %s %d,%d %dx%d dpr=%g\n", qPrintable(screen->name()), g.x(), g.y(), g.width(), g.height(), screen->devicePixelRatio());
	}

	printf("windows: %lld\n", static_cast<long long>(tracker->windows().length()));
	for (auto* w: tracker->windows()) dumpWindow(" ", w);

	auto* active = tracker->activeWindow();
	printf("active: %s\n", active ? qPrintable(active->title()) : "(none)");
	fflush(stdout);
}

}

int main(int argc, char** argv) {
	QLoggingCategory::setFilterRules("quickshell.windows.*.debug=true");
	QGuiApplication app(argc, argv);

	auto seconds = argc > 1 ? atoi(argv[1]) : 10;
	auto doSwitch = argc > 2 && QString(argv[2]) == "switch";
	auto doRemove = argc > 2 && QString(argv[2]) == "remove";

	auto* tracker = WindowTracker::instance();
	auto* vd = tracker->desktops();
	dump(tracker);

	QObject::connect(tracker, &WindowTracker::windowAdded, [](TrackedWindow* w) { dumpWindow("+", w); fflush(stdout); });
	QObject::connect(tracker, &WindowTracker::windowRemoved, [](TrackedWindow* w) { dumpWindow("-", w); fflush(stdout); });
	QObject::connect(tracker, &WindowTracker::activeWindowChanged, [tracker]() {
		auto* w = tracker->activeWindow();
		printf("* active: %s\n", w ? qPrintable(w->title()) : "(none)");
		fflush(stdout);
	});
	QObject::connect(vd, &VirtualDesktops::currentChanged, [vd]() {
		printf("* current desktop: %lld\n", static_cast<long long>(vd->currentIndex()));
		dumpRegistry();
		fflush(stdout);
	});
	QObject::connect(vd, &VirtualDesktops::desktopsChanged, [vd]() {
		printf("* desktop list: %lld desktops\n", static_cast<long long>(vd->count()));
		fflush(stdout);
	});

	if (doSwitch) {
		QTimer::singleShot(1000, [vd]() {
			if (vd->count() < 2) {
				printf("creating a second desktop: %d\n", vd->ensureCount(2));
			}
			auto target = (vd->currentIndex() + 1) % vd->count();
			printf("switching to desktop %lld: %d\n", static_cast<long long>(target), vd->switchTo(target));
			fflush(stdout);
		});

		QTimer::singleShot(3500, [vd, before = vd->currentIndex()]() {
			printf("current is now %lld; switching back to %lld: %d\n", static_cast<long long>(vd->currentIndex()), static_cast<long long>(before), vd->switchTo(before));
			fflush(stdout);
		});
	}

	if (doRemove) {
		QTimer::singleShot(1000, [vd, tracker]() {
			auto last = vd->count() - 1;
			auto occupied = false;
			for (auto* w: tracker->windows()) occupied = occupied || w->desktop() == last;

			if (vd->count() < 2 || occupied) {
				printf("not removing desktop %lld (count %lld, occupied %d)\n", static_cast<long long>(last), static_cast<long long>(vd->count()), occupied);
			} else {
				printf("removing desktop %lld: %d\n", static_cast<long long>(last), vd->removeDesktop(last, 0));
			}
			fflush(stdout);
		});
	}

	QTimer::singleShot(seconds * 1000, [tracker, &app]() {
		printf("--- final state\n");
		dump(tracker);
		app.quit();
	});

	return app.exec();
}
