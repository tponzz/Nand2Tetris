cmake_minimum_required(VERSION 3.16)

project(vm)

set(CMAKE_CXX_STANDARD 23)
set(CMAKE_CXX_STANDARD_REQUIRED ON)
set(CMAKE_CXX_EXTENSIONS OFF)

set(HEADER)
set(SRC
    vm.cpp
    code_writer.cpp
    parser.cpp
)

add_executable(vm ${SRC})
target_compile_options(vm PRIVATE -Wall -Wextra -Wswitch-enum)

add_subdirectory(test)
enable_testing()
