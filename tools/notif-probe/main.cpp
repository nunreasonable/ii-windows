// Console probe for Windows.UI.Notifications.Management.UserNotificationListener from an
// unpackaged exe. Commands:
//   notif-probe status
//   notif-probe request <timeoutSec>      RequestAccessAsync, gives up after the timeout
//   notif-probe list [logoDir]            GetNotificationsAsync(Toast), optionally saves logos
//   notif-probe events <sec>              try NotificationChanged, print events
//   notif-probe poll <sec>                poll GetNotificationsAsync every 2 s, print diffs
//   notif-probe remove <id>               RemoveNotification(id), then check it is gone
#include <windows.h>

#include <chrono>
#include <cstdio>
#include <set>
#include <string>
#include <thread>
#include <vector>

#include <winrt/Windows.ApplicationModel.h>
#include <winrt/Windows.Foundation.Collections.h>
#include <winrt/Windows.Foundation.h>
#include <winrt/Windows.Storage.Streams.h>
#include <winrt/Windows.UI.Notifications.Management.h>
#include <winrt/Windows.UI.Notifications.h>
#include <winrt/base.h>

using namespace winrt;
using namespace winrt::Windows::UI::Notifications;
using namespace winrt::Windows::UI::Notifications::Management;
using namespace winrt::Windows::Storage::Streams;

namespace {

const char* statusName(UserNotificationListenerAccessStatus s) {
	switch (s) {
	case UserNotificationListenerAccessStatus::Allowed: return "Allowed";
	case UserNotificationListenerAccessStatus::Denied: return "Denied";
	case UserNotificationListenerAccessStatus::Unspecified: return "Unspecified";
	}
	return "?";
}

std::string u8(const hstring& s) { return to_string(s); }

void printError(const char* what, const hresult_error& e) {
	printf("%s failed: 0x%08x %s\n", what, static_cast<uint32_t>(e.code().value), u8(e.message()).c_str());
}

void dumpOne(const UserNotification& n, const char* logoDir) {
	printf("- id=%u", n.Id());
	try {
		auto created = n.CreationTime();
		auto secs = std::chrono::duration_cast<std::chrono::seconds>(
		    winrt::clock::to_sys(created).time_since_epoch()
		).count();
		printf(" created=%lld", static_cast<long long>(secs));
	} catch (const hresult_error& e) { printError(" CreationTime", e); }

	try {
		auto info = n.AppInfo();
		printf(" aumid=\"%s\"", u8(info.AppUserModelId()).c_str());
		try { printf(" appId=\"%s\"", u8(info.Id()).c_str()); } catch (...) { printf(" appId=(err)"); }
		try { printf(" pfn=\"%s\"", u8(info.PackageFamilyName()).c_str()); } catch (...) { printf(" pfn=(err)"); }
		auto display = info.DisplayInfo();
		printf(" name=\"%s\"\n", u8(display.DisplayName()).c_str());

		if (logoDir != nullptr) {
			try {
				auto ref = display.GetLogo(winrt::Windows::Foundation::Size(64, 64));
				if (!ref) { printf("  logo: GetLogo returned null\n"); fflush(stdout); return; }
				auto stream = ref.OpenReadAsync().get();
				auto size = static_cast<uint32_t>(stream.Size());
				printf("  logo: %u bytes, type \"%s\"", size, u8(stream.ContentType()).c_str());
				DataReader reader(stream);
				reader.LoadAsync(size).get();
				std::vector<uint8_t> bytes(size);
				reader.ReadBytes(bytes);
				auto path = std::string(logoDir) + "\\logo-" + std::to_string(n.Id()) + ".bin";
				if (auto* f = fopen(path.c_str(), "wb")) {
					fwrite(bytes.data(), 1, bytes.size(), f);
					fclose(f);
					printf(" -> %s", path.c_str());
				}
				printf("\n");
			} catch (const hresult_error& e) { printError("  GetLogo", e); }
		}
	} catch (const hresult_error& e) { printError(" AppInfo", e); }

	try {
		auto toast = n.Notification();
		auto visual = toast.Visual();
		printf("  bindings=%u", visual.Bindings().Size());
		try {
			auto exp = toast.ExpirationTime();
			if (exp) printf(" expires=%lld", static_cast<long long>(std::chrono::duration_cast<std::chrono::seconds>(winrt::clock::to_sys(exp.Value()).time_since_epoch()).count()));
		} catch (...) {}
		printf("\n");
		for (auto const& binding: visual.Bindings()) {
			printf("  binding template=\"%s\" hints=%u\n", u8(binding.Template()).c_str(), binding.Hints().Size());
			for (auto const& hint: binding.Hints()) printf("    hint %s=%s\n", u8(hint.Key()).c_str(), u8(hint.Value()).c_str());
			for (auto const& text: binding.GetTextElements()) {
				printf("    text \"%s\"", u8(text.Text()).c_str());
				for (auto const& hint: text.Hints()) printf(" [%s=%s]", u8(hint.Key()).c_str(), u8(hint.Value()).c_str());
				printf("\n");
			}
		}
	} catch (const hresult_error& e) { printError("  Visual", e); }
}

int cmdStatus(const UserNotificationListener& listener) {
	printf("GetAccessStatus: %s\n", statusName(listener.GetAccessStatus()));
	return 0;
}

int cmdRequest(const UserNotificationListener& listener, int timeoutSec) {
	printf("before: %s\n", statusName(listener.GetAccessStatus()));
	fflush(stdout);
	auto op = listener.RequestAccessAsync();
	auto start = std::chrono::steady_clock::now();
	while (op.Status() == winrt::Windows::Foundation::AsyncStatus::Started) {
		if (std::chrono::steady_clock::now() - start > std::chrono::seconds(timeoutSec)) {
			printf("RequestAccessAsync still pending after %d s, giving up\n", timeoutSec);
			fflush(stdout);
			op.Cancel();
			return 2;
		}
		std::this_thread::sleep_for(std::chrono::milliseconds(100));
	}
	auto elapsed = std::chrono::duration_cast<std::chrono::milliseconds>(std::chrono::steady_clock::now() - start).count();
	try {
		printf("RequestAccessAsync -> %s after %lld ms\n", statusName(op.GetResults()), static_cast<long long>(elapsed));
	} catch (const hresult_error& e) { printError("RequestAccessAsync", e); }
	printf("after: %s\n", statusName(listener.GetAccessStatus()));
	return 0;
}

int cmdList(const UserNotificationListener& listener, const char* logoDir) {
	try {
		auto list = listener.GetNotificationsAsync(NotificationKinds::Toast).get();
		printf("GetNotificationsAsync(Toast): %u\n", list.Size());
		for (auto const& n: list) dumpOne(n, logoDir);
	} catch (const hresult_error& e) {
		printError("GetNotificationsAsync", e);
		return 1;
	}
	return 0;
}

int cmdEvents(const UserNotificationListener& listener, int secs) {
	try {
		auto token = listener.NotificationChanged([](auto&&, UserNotificationChangedEventArgs const& args) {
			printf("event kind=%d id=%u\n", static_cast<int>(args.ChangeKind()), args.UserNotificationId());
			fflush(stdout);
		});
		printf("NotificationChanged subscribed\n");
		fflush(stdout);
		std::this_thread::sleep_for(std::chrono::seconds(secs));
		listener.NotificationChanged(token);
	} catch (const hresult_error& e) {
		printError("NotificationChanged", e);
		return 1;
	}
	return 0;
}

int cmdPoll(const UserNotificationListener& listener, int secs) {
	std::set<uint32_t> known;
	auto first = true;
	auto end = std::chrono::steady_clock::now() + std::chrono::seconds(secs);
	while (std::chrono::steady_clock::now() < end) {
		auto t0 = std::chrono::steady_clock::now();
		try {
			auto list = listener.GetNotificationsAsync(NotificationKinds::Toast).get();
			auto dt = std::chrono::duration_cast<std::chrono::milliseconds>(std::chrono::steady_clock::now() - t0).count();
			std::set<uint32_t> now;
			for (auto const& n: list) {
				now.insert(n.Id());
				if (!first && !known.contains(n.Id())) {
					printf("added (%lld ms):\n", static_cast<long long>(dt));
					dumpOne(n, nullptr);
				}
			}
			for (auto id: known) if (!now.contains(id)) printf("removed %u\n", id);
			if (first) printf("baseline %zu ids (%lld ms)\n", now.size(), static_cast<long long>(dt));
			known = now;
			first = false;
		} catch (const hresult_error& e) { printError("GetNotificationsAsync", e); }
		fflush(stdout);
		std::this_thread::sleep_for(std::chrono::seconds(2));
	}
	return 0;
}

int cmdRemove(const UserNotificationListener& listener, uint32_t id) {
	try {
		auto before = listener.GetNotification(id);
		printf("GetNotification(%u) before: %s\n", id, before ? "present" : "null");
		listener.RemoveNotification(id);
		printf("RemoveNotification(%u) ok\n", id);
		auto after = listener.GetNotification(id);
		printf("GetNotification(%u) after: %s\n", id, after ? "present" : "null");
	} catch (const hresult_error& e) {
		printError("remove", e);
		return 1;
	}
	return 0;
}

} // namespace

int main(int argc, char** argv) {
	SetConsoleOutputCP(CP_UTF8);
	setvbuf(stdout, nullptr, _IOLBF, 4096);
	init_apartment(apartment_type::multi_threaded);

	std::string cmd = argc > 1 ? argv[1] : "status";

	UserNotificationListener listener {nullptr};
	try {
		listener = UserNotificationListener::Current();
	} catch (const hresult_error& e) {
		printError("UserNotificationListener::Current", e);
		return 1;
	}

	try {
		if (cmd == "status") return cmdStatus(listener);
		if (cmd == "request") return cmdRequest(listener, argc > 2 ? atoi(argv[2]) : 20);
		if (cmd == "list") return cmdList(listener, argc > 2 ? argv[2] : nullptr);
		if (cmd == "events") return cmdEvents(listener, argc > 2 ? atoi(argv[2]) : 10);
		if (cmd == "poll") return cmdPoll(listener, argc > 2 ? atoi(argv[2]) : 10);
		if (cmd == "remove" && argc > 2) return cmdRemove(listener, static_cast<uint32_t>(strtoul(argv[2], nullptr, 10)));
	} catch (const hresult_error& e) {
		printError(cmd.c_str(), e);
		return 1;
	}

	printf("unknown command\n");
	return 1;
}
