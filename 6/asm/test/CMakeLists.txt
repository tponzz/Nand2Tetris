
cmake_minimum_required(VERSION 3.14)

project(asm_tests LANGUAGES CXX)

enable_testing()

include(FetchContent)
FetchContent_Declare(
    googletest
    URL https://github.com/google/googletest/archive/refs/tags/v1.14.0.zip
)

# Keep gtest as a local build (don't install system-wide)
set(gtest_force_shared_crt ON CACHE BOOL "" FORCE)
FetchContent_MakeAvailable(googletest)

add_executable(asm_tests
    tst_parser.cpp
    tst_code.cpp
    tst_symbol_table.cpp
    ../parser.cpp
    ../code.cpp
    ../symbol_table.cpp
)

target_link_libraries(asm_tests PRIVATE gtest_main)

include(GoogleTest)
gtest_discover_tests(asm_tests)

