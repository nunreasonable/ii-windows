cmake_minimum_required(VERSION 3.20)
project(iiw_hypr_probe LANGUAGES CXX)
set(CMAKE_CXX_STANDARD 20)
set(QS_WINDOWS_SRC "" CACHE PATH "quickshell src/windows directory")
find_package(Qt6 6.11 REQUIRED COMPONENTS Gui)
qt_standard_project_setup(REQUIRES 6.11)
qt_add_executable(iiw_hypr_probe
	main.cpp
	${QS_WINDOWS_SRC}/util.cpp
	${QS_WINDOWS_SRC}/virtual_desktops.cpp
	${QS_WINDOWS_SRC}/virtual_desktops.hpp
	${QS_WINDOWS_SRC}/window_tracker.cpp
	${QS_WINDOWS_SRC}/window_tracker.hpp
)
target_include_directories(iiw_hypr_probe PRIVATE ${QS_WINDOWS_SRC})
target_compile_definitions(iiw_hypr_probe PRIVATE NOMINMAX WIN32_LEAN_AND_MEAN)
target_link_libraries(iiw_hypr_probe PRIVATE Qt6::Gui user32 shell32 dwmapi ole32 advapi32)
