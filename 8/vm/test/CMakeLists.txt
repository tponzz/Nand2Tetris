
cmake_minimum_required(VERSION 3.14)

project(vm_tests LANGUAGES CXX)

enable_testing()

include(FetchContent)
FetchContent_Declare(
  googletest
  GIT_REPOSITORY https://github.com/google/googletest.git
  GIT_TAG v1.17.0
  DOWNLOAD_EXTRACT_TIMESTAMP TRUE
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
