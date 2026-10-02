# Console probe for Windows.UI.Notifications.Management.UserNotificationListener from an
# unpackaged exe (no Qt): access status, RequestAccessAsync, listing/removing toasts, whether
# NotificationChanged works, polling diffs. Build and stage:
#   cmake -S tools/notif-probe -B build/notif-probe -G Ninja -DCMAKE_BUILD_TYPE=Release \
#     -DCMAKE_TOOLCHAIN_FILE=$IIW/tools/clang-cl-xwin.cmake
#   cmake --build build/notif-probe
#   mkdir -p dist/notif-probe && cp build/notif-probe/notif-probe.exe dist/notif-probe/
#   tools/vm.sh push notif-probe
cmake_minimum_required(VERSION 3.20)
project(notif_probe LANGUAGES CXX)
set(CMAKE_CXX_STANDARD 20)
add_executable(notif-probe main.cpp)
target_compile_definitions(notif-probe PRIVATE NOMINMAX WIN32_LEAN_AND_MEAN UNICODE _UNICODE _CRT_SECURE_NO_WARNINGS)
target_link_libraries(notif-probe PRIVATE windowsapp runtimeobject ole32 shell32)
