# Host (Linux) test for the blur shape walk of the Windows backend (src/windows/blur_shapes.cpp
# of the quickshell checkout given as QS_WINDOWS_SRC, which has no Windows dependencies). Loads
# small QML scenes shaped like ii's panels and checks the rectangles blur would go behind.
#   . tools/env.sh
#   cmake -S tools/blur-shapes-test -B build/blur-shapes-test -G Ninja \
#     -DCMAKE_PREFIX_PATH=$QT_HOST -DQS_WINDOWS_SRC=$IIW/wt-blur/src/windows
#   cmake --build build/blur-shapes-test && QT_QPA_PLATFORM=offscreen build/blur-shapes-test/blur_shapes_test
cmake_minimum_required(VERSION 3.20)
project(iiw_blur_shapes_test LANGUAGES CXX)
set(CMAKE_CXX_STANDARD 20)
set(QS_WINDOWS_SRC "" CACHE PATH "quickshell src/windows directory")
find_package(Qt6 6.11 REQUIRED COMPONENTS Quick QuickPrivate Qml)
qt_standard_project_setup(REQUIRES 6.11)
qt_add_executable(blur_shapes_test
	main.cpp
	${QS_WINDOWS_SRC}/blur_shapes.cpp
	${QS_WINDOWS_SRC}/blur_shapes.hpp
)
target_include_directories(blur_shapes_test PRIVATE ${QS_WINDOWS_SRC})
target_link_libraries(blur_shapes_test PRIVATE Qt6::Quick Qt6::QuickPrivate Qt6::Qml)
