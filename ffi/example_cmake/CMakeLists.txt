cmake_minimum_required(VERSION 3.16)
project(dee_config_gen_ffi_example C)

set(CMAKE_C_STANDARD 11)
set(CMAKE_C_STANDARD_REQUIRED ON)
set(CMAKE_C_EXTENSIONS OFF)

if(NOT DEFINED DCG_INCLUDE_DIR OR "${DCG_INCLUDE_DIR}" STREQUAL "")
  message(FATAL_ERROR "DCG_INCLUDE_DIR is required")
endif()

if(NOT DEFINED DCG_LIBRARY OR "${DCG_LIBRARY}" STREQUAL "")
  message(FATAL_ERROR "DCG_LIBRARY is required")
endif()

get_filename_component(DCG_INCLUDE_DIR "${DCG_INCLUDE_DIR}" ABSOLUTE)
get_filename_component(DCG_LIBRARY "${DCG_LIBRARY}" ABSOLUTE)
get_filename_component(DCG_LIBRARY_DIR "${DCG_LIBRARY}" DIRECTORY)

add_executable(dee_config_gen_ffi_example main.c)
target_include_directories(dee_config_gen_ffi_example PRIVATE "${DCG_INCLUDE_DIR}")

if(WIN32)
  add_library(dcg_ffi_lib SHARED IMPORTED GLOBAL)
  set_target_properties(dcg_ffi_lib PROPERTIES IMPORTED_IMPLIB "${DCG_LIBRARY}")
  if(DEFINED DCG_DLL AND NOT "${DCG_DLL}" STREQUAL "")
    get_filename_component(DCG_DLL "${DCG_DLL}" ABSOLUTE)
    set_target_properties(dcg_ffi_lib PROPERTIES IMPORTED_LOCATION "${DCG_DLL}")
  endif()
  target_link_libraries(dee_config_gen_ffi_example PRIVATE dcg_ffi_lib)
else()
  target_link_libraries(dee_config_gen_ffi_example PRIVATE "${DCG_LIBRARY}")
  set_target_properties(
    dee_config_gen_ffi_example
    PROPERTIES BUILD_RPATH "${DCG_LIBRARY_DIR}"
  )
endif()

include(CTest)
if(BUILD_TESTING)
  add_test(NAME run_ffi_example COMMAND dee_config_gen_ffi_example)
endif()
