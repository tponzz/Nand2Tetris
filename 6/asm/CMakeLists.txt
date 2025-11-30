cmake_minimum_required(VERSION 3.10)

project(asm_example VERSION 0.1 LANGUAGES CXX)

# Use a modern C++ standard
set(CMAKE_CXX_STANDARD 20)
set(CMAKE_CXX_STANDARD_REQUIRED ON)

# Produce a simple executable from single source file
add_library(parser STATIC parser.cpp)
add_library(code STATIC code.cpp)

add_executable(hackasm hackasm.cpp)

target_link_libraries(hackasm
    parser
    code
)

add_subdirectory(test)
enable_testing()