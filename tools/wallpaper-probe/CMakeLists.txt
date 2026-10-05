cmake_minimum_required(VERSION 3.20)
project(iiw_wallpaper_probe LANGUAGES CXX)
set(CMAKE_CXX_STANDARD 20)
set(QS_WINDOWS_SRC "" CACHE PATH "quickshell src/windows directory")
find_package(Qt6 6.11 REQUIRED COMPONENTS Gui QmlIntegration)
qt_standard_project_setup(REQUIRES 6.11)
qt_add_executable(iiw_wallpaper_probe
	main.cpp
	${QS_WINDOWS_SRC}/system/wallpaper.cpp
	${QS_WINDOWS_SRC}/system/wallpaper.hpp
)
target_include_directories(iiw_wallpaper_probe PRIVATE ${QS_WINDOWS_SRC}/system)
target_compile_definitions(iiw_wallpaper_probe PRIVATE NOMINMAX WIN32_LEAN_AND_MEAN)
target_link_libraries(iiw_wallpaper_probe PRIVATE Qt6::Gui Qt6::QmlIntegration user32 shell32 ole32 oleaut32 uuid advapi32)
