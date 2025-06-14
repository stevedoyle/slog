cmake_minimum_required(VERSION 3.30)

project(
  Cpp20Learning
  VERSION 1.0
  DESCRIPTION "A project for experimenting with CMake and C++20 features"
  LANGUAGES CXX)

set(CMAKE_CXX_STANDARD 20)
set(CMAKE_CXX_STANDARD_REQUIRED ON)
set(CMAKE_CXX_EXTENSIONS OFF)

include(CTest)

if(MSVC)
  # warning level 4
  add_compile_options(/W4)
else()
  add_compile_options(-Wall -Wextra -Wpedantic)
endif()

add_executable(cpp20_compare cpp20_compare.cpp)
add_test(NAME cpp20_compare_test COMMAND cpp20_compare)

add_executable(cpp20_for cpp20_for.cpp)
add_test(NAME cpp20_for_test COMMAND cpp20_for)

add_executable(cpp20_lambda cpp20_lambda.cpp)
add_test(NAME cpp20_lambda_test COMMAND cpp20_lambda)

add_executable(cpp20_enum cpp20_enum.cpp)
add_test(NAME cpp20_enum_test COMMAND cpp20_enum)

add_executable(cpp20_format cpp20_format.cpp)
add_test(NAME cpp20_format_test COMMAND cpp20_format)

add_executable(cpp20_span cpp20_span.cpp)
add_test(NAME cpp20_span_test COMMAND cpp20_span)

add_executable(cpp20_bit cpp20_bit.cpp)
add_test(NAME cpp20_bit_test COMMAND cpp20_bit)

add_executable(cpp20_coroutine cpp20_coroutine.cpp)
add_test(NAME cpp20_coroutine_test COMMAND cpp20_coroutine)
