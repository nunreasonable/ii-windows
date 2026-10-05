cmake_minimum_required(VERSION 3.20)
project(iiw_net_probe LANGUAGES CXX)
set(CMAKE_CXX_STANDARD 20)
set(QS_WINDOWS_SRC "" CACHE PATH "quickshell src/windows directory")
set(QS_CORE_SRC "" CACHE PATH "quickshell src/core directory")
find_package(Qt6 6.11 REQUIRED COMPONENTS Core Qml)
qt_standard_project_setup(REQUIRES 6.11)

qt_add_executable(iiw_net_probe
	main.cpp
	${QS_WINDOWS_SRC}/system/network.cpp
	${QS_WINDOWS_SRC}/system/network.hpp
	${QS_WINDOWS_SRC}/system/network_wifi.cpp
	${QS_WINDOWS_SRC}/system/network_wifi.hpp
	${QS_WINDOWS_SRC}/system/network_wifi_item.cpp
	${QS_WINDOWS_SRC}/system/network_wifi_item.hpp
	${QS_WINDOWS_SRC}/system/network_connectivity.cpp
	${QS_WINDOWS_SRC}/system/network_connectivity.hpp
	${QS_CORE_SRC}/model.cpp
)
target_include_directories(iiw_net_probe PRIVATE ${QS_WINDOWS_SRC} ${QS_WINDOWS_SRC}/system ${QS_CORE_SRC}/..)
target_compile_definitions(iiw_net_probe PRIVATE NOMINMAX WIN32_LEAN_AND_MEAN)
target_link_libraries(iiw_net_probe PRIVATE Qt6::Core Qt6::Qml ole32 oleaut32 uuid wlanapi iphlpapi)
