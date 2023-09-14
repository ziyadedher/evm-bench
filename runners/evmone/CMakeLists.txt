cmake_minimum_required(VERSION 3.14)

# Use CPM for CMake dependency management
include(cmake/CPM.cmake)

CPMAddPackage("gh:CLIUtils/CLI11@2.3.2")
CPMAddPackage("gh:chfast/intx@0.10.1")
CPMAddPackage("gh:ethereum/evmone@0.10.0")

project(runner-evmone LANGUAGES CXX C)

add_executable(runner-evmone runner.cpp)
target_link_libraries(runner-evmone PRIVATE CLI11::CLI11 evmone)
