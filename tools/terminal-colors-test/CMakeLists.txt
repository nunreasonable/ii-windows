# Checks the Windows backend's terminal color harmonization (src/windows/system/
# terminal_colors_math.cpp + third_party/material_color of the quickshell checkout given as
# QS_WINDOWS_SRC; no Qt) against ii's scripts/colors/generate_colors_material.py.
# On the host (run.sh does these steps):
#   cmake -S tools/terminal-colors-test -B build/terminal-colors-test -G Ninja \
#     -DQS_WINDOWS_SRC=$IIW/wt-term/src/windows
#   cmake --build build/terminal-colors-test
#   ~/.local/state/quickshell/.venv/bin/python tools/terminal-colors-test/compare.py \
#     build/terminal-colors-test/terminal_colors_test
# For Windows (same libm the shell gets), add -DCMAKE_TOOLCHAIN_FILE=$IIW/tools/clang-cl-xwin.cmake
# and run compare.py --write <dir> to get cases.txt/expected.txt to diff against on the target.
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

# The shell is built without FMA contraction (x86-64 baseline, no fast-math); keep host builds
# the same even with -march flags in the environment.
if (NOT MSVC)
	target_compile_options(terminal_colors_test PRIVATE -ffp-contract=off)
endif()
