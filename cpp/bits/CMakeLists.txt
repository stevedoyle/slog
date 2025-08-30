cmake_minimum_required(VERSION 3.16)

project(maxbits LANGUAGES CXX)

set(CMAKE_CXX_STANDARD 17)
set(CMAKE_CXX_STANDARD_REQUIRED ON)

# Add compiler flags for optimization and warnings
set(CMAKE_CXX_FLAGS_RELEASE "-O3 -DNDEBUG")
set(CMAKE_CXX_FLAGS_DEBUG "-g -O0")

if(NOT CMAKE_BUILD_TYPE)
    set(CMAKE_BUILD_TYPE Release)
endif()

# Find or fetch Google Benchmark
find_package(benchmark QUIET)
if(NOT benchmark_FOUND)
    include(FetchContent)
    FetchContent_Declare(
        googlebenchmark
        GIT_REPOSITORY https://github.com/google/benchmark.git
        GIT_TAG v1.8.3
    )
    set(BENCHMARK_ENABLE_TESTING OFF)
    FetchContent_MakeAvailable(googlebenchmark)
endif()

# Find or fetch Google Test
find_package(GTest QUIET)
if(NOT GTest_FOUND)
    include(FetchContent)
    FetchContent_Declare(
        googletest
        GIT_REPOSITORY https://github.com/google/googletest.git
        GIT_TAG v1.14.0
    )
    set(gtest_force_shared_crt ON CACHE BOOL "" FORCE)
    FetchContent_MakeAvailable(googletest)
endif()

# Create a library from maxbits.cpp
add_library(maxbits STATIC maxbits.cpp)

# Create benchmark executable
add_executable(maxbits_benchmark benchmark.cpp)
target_link_libraries(maxbits_benchmark maxbits benchmark::benchmark benchmark::benchmark_main)

# Create test executable
add_executable(maxbits_test test.cpp)
target_link_libraries(maxbits_test maxbits gtest gtest_main)

# Enable testing
enable_testing()
add_test(NAME maxbits_unit_tests COMMAND maxbits_test)
