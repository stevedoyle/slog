# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Build System

This is a CMake-based C++17 project that uses Google Benchmark and Google Test frameworks. The CMake configuration automatically fetches dependencies if they're not found locally.

### Core Build Commands

```bash
# Standard build workflow (from project root)
mkdir build && cd build
cmake ..
make

# Or use cmake directly
cmake --build .
```

### Available Targets

- `maxbits` - Static library containing the core functionality
- `maxbits_benchmark` - Google Benchmark executable for performance testing
- `maxbits_test` - Google Test executable for unit testing

### Running Tests and Benchmarks

```bash
# Run unit tests
./maxbits_test                    # Direct execution
ctest --output-on-failure        # Via CMake test runner

# Run benchmarks
./maxbits_benchmark                                        # All benchmarks
./maxbits_benchmark --benchmark_filter="Small8|AllZeros"  # Filtered benchmarks
```

## Code Architecture

### Core Library (`maxbits.cpp` + `maxbits.h`)

The main functionality centers around computing the minimum number of bits needed to represent the maximum value in arrays of different integer types.

**Key Functions:**
- `computeNumBits()` - Main API that dispatches to type-specific implementations
- `computeMaxValue{8,16,32,64}()` - Type-specific maximum value finders
- `highbit64()` - Utility to find the position of the highest set bit

**Function Signature Pattern:**
- Functions expect direct pointers to data arrays, not pointer-to-pointer
- Element width parameter determines which type-specific function to use (1, 2, 4, or 8 bytes)
- Returns minimum bits needed, with special case of returning 1 for all-zero arrays

### Testing Strategy

The test suite (`test.cpp`) follows a comprehensive approach:
- **Unit tests** for each individual function including edge cases
- **Integration tests** for the main `computeNumBits()` function with various data types
- **Boundary testing** for powers of 2, empty arrays, and maximum values
- **Performance validation** through large array tests

### Benchmark Design

The benchmark suite (`benchmark.cpp`) includes:
- **Parameterized benchmarks** across different array sizes (8 to 262K elements)
- **Type-specific benchmarks** for 8, 16, 32, and 64-bit integers
- **Special case benchmarks** for common scenarios (small arrays, all zeros, max values)
- **Reproducible results** using fixed random seeds

## Development Notes

- The project defaults to Release build mode for performance testing
- Functions previously marked `static` have been exposed in the header for comprehensive testing
- CMake automatically handles dependency management for Google Test and Benchmark
- The codebase assumes C++17 standard and uses Clang compiler optimizations
