
cmake_minimum_required(VERSION 3.14)

project(vm_tests LANGUAGES CXX)

enable_testing()

include(FetchContent)
FetchContent_Declare(
    googletest
    URL https://github.com/google/googletest/archive/refs/tags/v1.14.0.zip
)

# Keep gtest as a local build (don't install system-wide)
set(gtest_force_shared_crt ON CACHE BOOL "" FORCE)
FetchContent_MakeAvailable(googletest)

add_executable(${PROJECT_NAME}
    tst_parser.cpp
    ../parser.cpp
)

target_link_libraries(${PROJECT_NAME}
    PRIVATE gtest_main)

include(GoogleTest)
gtest_discover_tests(${PROJECT_NAME})