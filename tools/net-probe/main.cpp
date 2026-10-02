// Console probe for qs::windows::sys::Network: dumps connectivity/ethernet/wifi state, watches
// every change for a while, and issues one explicit rescan.
//   iiw_net_probe [seconds]
//
// No logging.cpp pulled in from the real tree (that cascades into much of the "Quickshell" core
// library for no benefit here) -- this is a trivial stand-in for qs::log::initLogCategoryLevel
// (called by the QS_LOGGING_CATEGORY macro in network_wifi.cpp/network_connectivity.cpp) so
// their qCWarning/qCDebug calls still print, just without the real rules-file/env-var level
// overrides.
#include <cstdio>

#include <qcoreapplication.h>
#include <qloggingcategory.h>
#include <qobject.h>
#include <qtimer.h>

#include "network.hpp"

namespace qs::log {
void initLogCategoryLevel(const char* /*name*/, QtMsgType /*defaultLevel*/) {}
}

using qs::windows::sys::Network;

namespace {

void dump(Network* net) {
	printf(
	    "hasInternet=%d ethernet=%d wifiAdapter=%d wifiRadioOn=%d wifiScanning=%d "
	    "wifiConnecting=%d needsLocationPermission=%d wifiStatus=%s\n",
	    net->property("hasInternet").toBool(),
	    net->property("ethernetConnected").toBool(),
	    net->property("wifiAdapterPresent").toBool(),
	    net->property("wifiRadioOn").toBool(),
	    net->property("wifiScanning").toBool(),
	    net->property("wifiConnecting").toBool(),
	    net->property("needsLocationPermission").toBool(),
	    qPrintable(net->property("wifiStatus").toString())
	);
	printf(
	    "  active ssid=\"%s\" bssid=%s signal=%d security=\"%s\"\n",
	    qPrintable(net->property("activeSsid").toString()),
	    qPrintable(net->property("activeBssid").toString()),
	    net->property("activeSignalQuality").toInt(),
	    qPrintable(net->property("activeSecurity").toString())
	);

	auto* model = net->networks();
	auto values = model->values();
	printf("  networks: %lld\n", static_cast<long long>(values.length()));
	for (auto* obj: values) {
		printf(
		    "    ssid=\"%s\" bssid=%s strength=%d freqMHz=%d active=%d security=\"%s\" hasProfile=%d\n",
		    qPrintable(obj->property("ssid").toString()),
		    qPrintable(obj->property("bssid").toString()),
		    obj->property("strength").toInt(),
		    obj->property("frequency").toInt(),
		    obj->property("active").toBool(),
		    qPrintable(obj->property("security").toString()),
		    obj->property("hasProfile").toBool()
		);
	}
	fflush(stdout);
}

} // namespace

int main(int argc, char** argv) {
	QLoggingCategory::setFilterRules("quickshell.windows.network.*.debug=true");
	QCoreApplication app(argc, argv);

	auto seconds = argc > 1 ? atoi(argv[1]) : 8;

	Network network;
	printf("--- initial state\n");
	dump(&network);

	QObject::connect(&network, &Network::wifiStatusChanged, [&network]() {
		printf("* wifiStatus -> %s\n", qPrintable(network.property("wifiStatus").toString()));
		fflush(stdout);
	});
	QObject::connect(&network, &Network::ethernetConnectedChanged, [&network]() {
		printf("* ethernetConnected -> %d\n", network.property("ethernetConnected").toBool());
		fflush(stdout);
	});
	QObject::connect(&network, &Network::wifiAdapterPresentChanged, [&network]() {
		printf("* wifiAdapterPresent -> %d\n", network.property("wifiAdapterPresent").toBool());
		fflush(stdout);
	});
	QObject::connect(&network, &Network::needsLocationPermissionChanged, [&network]() {
		printf("* needsLocationPermission -> %d\n", network.property("needsLocationPermission").toBool());
		fflush(stdout);
	});
	QObject::connect(network.networks(), &UntypedObjectModel::valuesChanged, [&network]() {
		printf("* networks changed, now %lld\n", static_cast<long long>(network.networks()->values().length()));
		fflush(stdout);
	});

	QTimer::singleShot(1500, [&network]() {
		printf("--- requesting scan\n");
		fflush(stdout);
		network.scanWifiNetworks();
	});

	QTimer::singleShot(seconds * 1000, [&network, &app]() {
		printf("--- final state\n");
		dump(&network);
		app.quit();
	});

	return app.exec();
}
