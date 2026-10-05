cmake_minimum_required(VERSION 3.20)
project(iiw_terminal_colors_test LANGUAGES CXX)
set(CMAKE_CXX_STANDARD 20)
set(CMAKE_CXX_STANDARD_REQUIRED ON)
if (NOT CMAKE_BUILD_TYPE)
	set(CMAKE_BUILD_TYPE RelWithDebInfo)
endif()

set(QS_WINDOWS_SRC "" CACHE PATH "quickshell src/windows directory")
if (NOT EXISTS "${QS_WINDOWS_SRC}/system/terminal_colors_math.cpp")
	message(FATAL_ERROR "QS_WINDOWS_SRC must point at a quickshell checkout's src/windows")
endif()

set(MCU "${QS_WINDOWS_SRC}/third_party/material_color")
add_executable(terminal_colors_test
	main.cpp
	${QS_WINDOWS_SRC}/system/terminal_colors_math.cpp
	${MCU}/cpp/cam/cam.cc
	${MCU}/cpp/cam/hct.cc
	${MCU}/cpp/cam/hct_solver.cc
	${MCU}/cpp/utils/utils.cc
)
target_include_directories(terminal_colors_test PRIVATE ${QS_WINDOWS_SRC}/system ${MCU})

if (NOT MSVC)
	target_compile_options(terminal_colors_test PRIVATE -ffp-contract=off)
endif()
