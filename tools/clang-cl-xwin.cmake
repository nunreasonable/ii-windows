# Cross toolchain: Linux host -> x86_64 Windows (MSVC ABI) with clang-cl + lld-link,
# MSVC CRT / Windows SDK from `xwin splat`, Qt from aqt (msvc2022_64) with a matching host Qt.

set(CMAKE_SYSTEM_NAME Windows)
set(CMAKE_SYSTEM_VERSION 10.0.26100)
set(CMAKE_SYSTEM_PROCESSOR AMD64)

get_filename_component(IIW "${CMAKE_CURRENT_LIST_DIR}/.." ABSOLUTE)
set(XWIN "${IIW}/toolchain/xwin")
set(QT_WIN "${IIW}/toolchain/qt/6.11.2/msvc2022_64")
set(QT_HOST "${IIW}/toolchain/qt/6.11.2/gcc_64")

set(CMAKE_C_COMPILER clang-cl)
set(CMAKE_CXX_COMPILER clang-cl)
set(CMAKE_C_COMPILER_TARGET x86_64-pc-windows-msvc)
set(CMAKE_CXX_COMPILER_TARGET x86_64-pc-windows-msvc)

find_program(IIW_LLD_LINK lld-link PATHS /usr/bin "${IIW}/toolchain/bin" REQUIRED NO_DEFAULT_PATH)
set(CMAKE_LINKER "${IIW_LLD_LINK}")
find_program(IIW_LLVM_LIB NAMES llvm-lib PATHS /usr/bin "${IIW}/toolchain/bin" NO_DEFAULT_PATH)
if(IIW_LLVM_LIB)
	set(CMAKE_AR "${IIW_LLVM_LIB}")
else()
	# lld-link /lib works as a librarian when llvm-lib is missing.
	set(CMAKE_AR "${IIW_LLD_LINK}")
	set(CMAKE_CXX_CREATE_STATIC_LIBRARY "<CMAKE_AR> /lib /nologo <LINK_FLAGS> /out:<TARGET> <OBJECTS>")
	set(CMAKE_C_CREATE_STATIC_LIBRARY "<CMAKE_AR> /lib /nologo <LINK_FLAGS> /out:<TARGET> <OBJECTS>")
endif()
find_program(CMAKE_RC_COMPILER NAMES llvm-rc PATHS /usr/bin "${IIW}/toolchain/bin" NO_DEFAULT_PATH)
find_program(CMAKE_MT NAMES llvm-mt PATHS /usr/bin "${IIW}/toolchain/bin" NO_DEFAULT_PATH)

set(_iiw_inc
	"/imsvc${XWIN}/crt/include"
	"/imsvc${XWIN}/sdk/include/ucrt"
	"/imsvc${XWIN}/sdk/include/um"
	"/imsvc${XWIN}/sdk/include/shared"
	"/imsvc${XWIN}/sdk/include/winrt"
	"/imsvc${XWIN}/sdk/include/cppwinrt"
)
list(JOIN _iiw_inc " " _iiw_inc)
set(CMAKE_C_FLAGS_INIT "${_iiw_inc}")
set(CMAKE_CXX_FLAGS_INIT "${_iiw_inc} /EHsc")
set(CMAKE_RC_FLAGS_INIT "-I${XWIN}/sdk/include/um -I${XWIN}/sdk/include/shared")

set(_iiw_libs
	"/libpath:${XWIN}/crt/lib/x86_64"
	"/libpath:${XWIN}/sdk/lib/um/x86_64"
	"/libpath:${XWIN}/sdk/lib/ucrt/x86_64"
)
list(JOIN _iiw_libs " " _iiw_libs)
set(CMAKE_EXE_LINKER_FLAGS_INIT "${_iiw_libs}")
set(CMAKE_SHARED_LINKER_FLAGS_INIT "${_iiw_libs}")
set(CMAKE_MODULE_LINKER_FLAGS_INIT "${_iiw_libs}")

# Link with lld-link directly instead of CMake's `vs_link_exe` wrapper, which needs rc.exe/mt.exe
# for manifests; lld-link embeds the default manifest by itself.
string(APPEND CMAKE_EXE_LINKER_FLAGS_INIT " /manifest:embed")
set(CMAKE_USER_MAKE_RULES_OVERRIDE "${CMAKE_CURRENT_LIST_DIR}/clang-cl-rules.cmake")

# Qt is built against the dynamic release CRT.
set(CMAKE_MSVC_RUNTIME_LIBRARY "MultiThreadedDLL")

set(QT_HOST_PATH "${QT_HOST}" CACHE PATH "")
set(QT_HOST_PATH_CMAKE_DIR "${QT_HOST}/lib/cmake" CACHE PATH "")
list(APPEND CMAKE_PREFIX_PATH "${QT_WIN}")
set(CMAKE_FIND_ROOT_PATH "${QT_WIN}" "${XWIN}")
set(CMAKE_FIND_ROOT_PATH_MODE_PROGRAM NEVER)
set(CMAKE_FIND_ROOT_PATH_MODE_LIBRARY ONLY)
set(CMAKE_FIND_ROOT_PATH_MODE_INCLUDE ONLY)
set(CMAKE_FIND_ROOT_PATH_MODE_PACKAGE ONLY)
