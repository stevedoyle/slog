#include <benchmark/benchmark.h>
#include <cstdint>
#include <random>
#include <vector>

// Forward declaration of the function to benchmark
int computeNumBits(const void *const src, size_t const numElements,
                   size_t const elementWidth);

using namespace benchmark;

class BenchmarkFixture : public benchmark::Fixture {
public:
  void SetUp(const ::benchmark::State &state) override {
    // Generate random data for different element sizes
    size_t numElements = state.range(0);

    // 8-bit data
    data8.resize(numElements);
    std::mt19937 gen(42); // Fixed seed for reproducible results
    std::uniform_int_distribution<uint8_t> dist8(0, 255);
    for (auto &val : data8) {
      val = dist8(gen);
    }

    // 16-bit data
    data16.resize(numElements);
    std::uniform_int_distribution<uint16_t> dist16(0, 65535);
    for (auto &val : data16) {
      val = dist16(gen);
    }

    // 32-bit data
    data32.resize(numElements);
    std::uniform_int_distribution<uint32_t> dist32(0, UINT32_MAX);
    for (auto &val : data32) {
      val = dist32(gen);
    }

    // 64-bit data
    data64.resize(numElements);
    std::uniform_int_distribution<uint64_t> dist64(0, UINT64_MAX);
    for (auto &val : data64) {
      val = dist64(gen);
    }
  }

protected:
  std::vector<uint8_t> data8;
  std::vector<uint16_t> data16;
  std::vector<uint32_t> data32;
  std::vector<uint64_t> data64;
};

BENCHMARK_DEFINE_F(BenchmarkFixture, ComputeNumBits8)(benchmark::State &state) {
  size_t numElements = state.range(0);
  const void *ptr = data8.data();

  for (auto _ : state) {
    int result = computeNumBits(ptr, numElements, 1);
    benchmark::DoNotOptimize(result);
  }
  state.SetItemsProcessed(state.iterations() * numElements);
}

BENCHMARK_DEFINE_F(BenchmarkFixture, ComputeNumBits16)
(benchmark::State &state) {
  size_t numElements = state.range(0);
  const void *ptr = data16.data();

  for (auto _ : state) {
    int result = computeNumBits(ptr, numElements, 2);
    benchmark::DoNotOptimize(result);
  }
  state.SetItemsProcessed(state.iterations() * numElements);
}

BENCHMARK_DEFINE_F(BenchmarkFixture, ComputeNumBits32)
(benchmark::State &state) {
  size_t numElements = state.range(0);
  const void *ptr = data32.data();

  for (auto _ : state) {
    int result = computeNumBits(ptr, numElements, 4);
    benchmark::DoNotOptimize(result);
  }
  state.SetItemsProcessed(state.iterations() * numElements);
}

BENCHMARK_DEFINE_F(BenchmarkFixture, ComputeNumBits64)
(benchmark::State &state) {
  size_t numElements = state.range(0);
  const void *ptr = data64.data();

  for (auto _ : state) {
    int result = computeNumBits(ptr, numElements, 8);
    benchmark::DoNotOptimize(result);
  }
  state.SetItemsProcessed(state.iterations() * numElements);
}

// Register benchmarks with various data sizes
BENCHMARK_REGISTER_F(BenchmarkFixture, ComputeNumBits8)
    ->Range(8, 8 << 15) // From 8 to 262,144 elements
    ->Unit(benchmark::kNanosecond);

BENCHMARK_REGISTER_F(BenchmarkFixture, ComputeNumBits16)
    ->Range(8, 8 << 15)
    ->Unit(benchmark::kNanosecond);

BENCHMARK_REGISTER_F(BenchmarkFixture, ComputeNumBits32)
    ->Range(8, 8 << 15)
    ->Unit(benchmark::kNanosecond);

BENCHMARK_REGISTER_F(BenchmarkFixture, ComputeNumBits64)
    ->Range(8, 8 << 15)
    ->Unit(benchmark::kNanosecond);

// Simple microbenchmarks for specific cases
static void BM_ComputeNumBits_Small8(benchmark::State &state) {
  std::vector<uint8_t> data = {1, 2, 3, 4, 5, 6, 7, 8};
  const void *ptr = data.data();

  for (auto _ : state) {
    int result = computeNumBits((const void *const *)&ptr, data.size(), 1);
    benchmark::DoNotOptimize(result);
  }
}
BENCHMARK(BM_ComputeNumBits_Small8);

static void BM_ComputeNumBits_AllZeros(benchmark::State &state) {
  std::vector<uint32_t> data(1000, 0);
  const void *ptr = data.data();

  for (auto _ : state) {
    int result = computeNumBits((const void *const *)&ptr, data.size(), 4);
    benchmark::DoNotOptimize(result);
  }
}
BENCHMARK(BM_ComputeNumBits_AllZeros);

static void BM_ComputeNumBits_MaxValues(benchmark::State &state) {
  std::vector<uint32_t> data(1000, UINT32_MAX);
  const void *ptr = data.data();

  for (auto _ : state) {
    int result = computeNumBits((const void *const *)&ptr, data.size(), 4);
    benchmark::DoNotOptimize(result);
  }
}
BENCHMARK(BM_ComputeNumBits_MaxValues);
