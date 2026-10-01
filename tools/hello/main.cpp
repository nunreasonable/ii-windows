// Toolchain smoke test: translucent Qt Quick window + a C++/WinRT call (GSMTC).
#include <QGuiApplication>
#include <QQmlApplicationEngine>
#include <QQmlContext>

#include <thread>

#include <winrt/Windows.Foundation.h>
#include <winrt/Windows.Foundation.Collections.h>
#include <winrt/Windows.Media.Control.h>

// Qt initializes COM as STA on the GUI thread, and C++/WinRT may not block (.get()) on an STA
// thread, so WinRT calls run on their own MTA thread.
static QString mediaSessionsOnMta() {
	try {
		winrt::init_apartment(winrt::apartment_type::multi_threaded);
		using namespace winrt::Windows::Media::Control;
		auto manager = GlobalSystemMediaTransportControlsSessionManager::RequestAsync().get();
		auto sessions = manager.GetSessions();
		return QStringLiteral("GSMTC ok, %1 media session(s)").arg(sessions.Size());
	} catch (const winrt::hresult_error& e) {
		return QStringLiteral("GSMTC error: ") + QString::fromWCharArray(e.message().c_str());
	}
}

static QString mediaSessions() {
	QString result;
	std::thread([&] { result = mediaSessionsOnMta(); }).join();
	return result;
}

int main(int argc, char* argv[]) {
	QGuiApplication app(argc, argv);
	QQmlApplicationEngine engine;
	engine.rootContext()->setContextProperty("winrtStatus", mediaSessions());
	engine.loadFromModule("Hello", "Main");
	return app.exec();
}
