cmake_minimum_required(VERSION 3.16)

project(vm)

set(CMAKE_CXX_STANDARD 20)
set(CMAKE_CXX_STANDARD_REQUIRED ON)
set(CMAKE_CXX_EXTENSIONS OFF)

set(HEADER)
set(SRC vm.cpp)

add_executable(vm ${SRC})

add_subdirectory(test)
enable_testing()