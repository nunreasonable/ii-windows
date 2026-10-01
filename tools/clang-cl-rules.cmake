# Loaded after the Windows-MSVC platform file (see clang-cl-xwin.cmake).
foreach(lang C CXX)
	set(CMAKE_${lang}_LINK_EXECUTABLE
		"<CMAKE_LINKER> /nologo <OBJECTS> /out:<TARGET> /implib:<TARGET_IMPLIB> /pdb:<TARGET_PDB> /version:<TARGET_VERSION_MAJOR>.<TARGET_VERSION_MINOR> <CMAKE_${lang}_LINK_FLAGS> <LINK_FLAGS> <LINK_LIBRARIES>")
	set(CMAKE_${lang}_CREATE_SHARED_LIBRARY
		"<CMAKE_LINKER> /nologo <OBJECTS> /out:<TARGET> /implib:<TARGET_IMPLIB> /pdb:<TARGET_PDB> /dll /version:<TARGET_VERSION_MAJOR>.<TARGET_VERSION_MINOR> <LINK_FLAGS> <LINK_LIBRARIES>")
	set(CMAKE_${lang}_CREATE_SHARED_MODULE "${CMAKE_${lang}_CREATE_SHARED_LIBRARY}")
endforeach()
