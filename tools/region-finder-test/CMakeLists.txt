cmake_minimum_required(VERSION 3.20)
project(iiw_region_finder_test LANGUAGES CXX)
set(CMAKE_CXX_STANDARD 20)
set(CMAKE_CXX_STANDARD_REQUIRED ON)
if (NOT CMAKE_BUILD_TYPE)
	set(CMAKE_BUILD_TYPE RelWithDebInfo)
endif()

set(QS_WINDOWS_SRC "" CACHE PATH "quickshell src/windows directory")
if (NOT EXISTS "${QS_WINDOWS_SRC}/image/region_finder.cpp")
	message(FATAL_ERROR "QS_WINDOWS_SRC must point at a quickshell checkout's src/windows")
endif()

find_package(Qt6 6.11 REQUIRED COMPONENTS Gui)
qt_standard_project_setup(REQUIRES 6.11)

qt_add_executable(region_finder_test
	main.cpp
	${QS_WINDOWS_SRC}/image/region_finder.cpp
	${QS_WINDOWS_SRC}/image/region_finder.hpp
)
target_include_directories(region_finder_test PRIVATE ${QS_WINDOWS_SRC}/image)
target_link_libraries(region_finder_test PRIVATE Qt6::Gui)
