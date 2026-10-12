cmake_minimum_required(VERSION 3.20)
project(iiw_recorder_audio_test LANGUAGES CXX)
set(CMAKE_CXX_STANDARD 20)
set(CMAKE_CXX_STANDARD_REQUIRED ON)
if (NOT CMAKE_BUILD_TYPE)
	set(CMAKE_BUILD_TYPE RelWithDebInfo)
endif()

set(QS_WINDOWS_SRC "" CACHE PATH "quickshell src/windows directory")
if (NOT EXISTS "${QS_WINDOWS_SRC}/recorder/audio_dsp.cpp")
	message(FATAL_ERROR "QS_WINDOWS_SRC must point at a quickshell checkout's src/windows")
endif()

add_executable(recorder_audio_test
	main.cpp
	${QS_WINDOWS_SRC}/recorder/audio_dsp.cpp
)
target_include_directories(recorder_audio_test PRIVATE ${QS_WINDOWS_SRC}/recorder)
if (NOT MSVC)
	target_compile_options(recorder_audio_test PRIVATE -Wall -Wextra)
endif()
