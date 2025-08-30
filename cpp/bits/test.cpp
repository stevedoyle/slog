#include "maxbits.h"
#include <climits>
#include <cstdint>
#include <gtest/gtest.h>
#include <vector>

class MaxBitsTest : public ::testing::Test {
protected:
  void SetUp() override {}
  void TearDown() override {}
};

// Tests for highbit64 function
TEST_F(MaxBitsTest, HighBit64_Zero) { EXPECT_EQ(highbit64(0), -1); }

TEST_F(MaxBitsTest, HighBit64_PowersOfTwo) {
  EXPECT_EQ(highbit64(1), 0);   // 2^0
  EXPECT_EQ(highbit64(2), 1);   // 2^1
  EXPECT_EQ(highbit64(4), 2);   // 2^2
  EXPECT_EQ(highbit64(8), 3);   // 2^3
  EXPECT_EQ(highbit64(16), 4);  // 2^4
  EXPECT_EQ(highbit64(32), 5);  // 2^5
  EXPECT_EQ(highbit64(64), 6);  // 2^6
  EXPECT_EQ(highbit64(128), 7); // 2^7
  EXPECT_EQ(highbit64(256), 8); // 2^8
}

TEST_F(MaxBitsTest, HighBit64_NonPowersOfTwo) {
  EXPECT_EQ(highbit64(3), 1);   // 011b -> highest bit at position 1
  EXPECT_EQ(highbit64(5), 2);   // 101b -> highest bit at position 2
  EXPECT_EQ(highbit64(7), 2);   // 111b -> highest bit at position 2
  EXPECT_EQ(highbit64(15), 3);  // 1111b -> highest bit at position 3
  EXPECT_EQ(highbit64(255), 7); // 11111111b -> highest bit at position 7
}

TEST_F(MaxBitsTest, HighBit64_LargeValues) {
  EXPECT_EQ(highbit64(UINT32_MAX), 31); // 2^32 - 1
  EXPECT_EQ(highbit64(UINT64_MAX), 63); // 2^64 - 1
}

// Tests for computeMaxValue8
TEST_F(MaxBitsTest, ComputeMaxValue8_EmptyArray) {
  uint8_t data[] = {};
  EXPECT_EQ(computeMaxValue8(data, 0), 0);
}

TEST_F(MaxBitsTest, ComputeMaxValue8_SingleElement) {
  uint8_t data[] = {42};
  EXPECT_EQ(computeMaxValue8(data, 1), 42);
}

TEST_F(MaxBitsTest, ComputeMaxValue8_MultipleElements) {
  uint8_t data[] = {1, 255, 100, 50, 200};
  EXPECT_EQ(computeMaxValue8(data, 5), 255);
}

TEST_F(MaxBitsTest, ComputeMaxValue8_AllZeros) {
  uint8_t data[] = {0, 0, 0, 0};
  EXPECT_EQ(computeMaxValue8(data, 4), 0);
}

// Tests for computeMaxValue16
TEST_F(MaxBitsTest, ComputeMaxValue16_EmptyArray) {
  uint16_t data[] = {};
  EXPECT_EQ(computeMaxValue16(data, 0), 0);
}

TEST_F(MaxBitsTest, ComputeMaxValue16_SingleElement) {
  uint16_t data[] = {1000};
  EXPECT_EQ(computeMaxValue16(data, 1), 1000);
}

TEST_F(MaxBitsTest, ComputeMaxValue16_MultipleElements) {
  uint16_t data[] = {1, 65535, 1000, 500, 2000};
  EXPECT_EQ(computeMaxValue16(data, 5), 65535);
}

// Tests for computeMaxValue32
TEST_F(MaxBitsTest, ComputeMaxValue32_EmptyArray) {
  uint32_t data[] = {};
  EXPECT_EQ(computeMaxValue32(data, 0), 0);
}

TEST_F(MaxBitsTest, ComputeMaxValue32_SingleElement) {
  uint32_t data[] = {100000};
  EXPECT_EQ(computeMaxValue32(data, 1), 100000);
}

TEST_F(MaxBitsTest, ComputeMaxValue32_MultipleElements) {
  uint32_t data[] = {1, UINT32_MAX, 1000, 500, 2000};
  EXPECT_EQ(computeMaxValue32(data, 5), UINT32_MAX);
}

// Tests for computeMaxValue64
TEST_F(MaxBitsTest, ComputeMaxValue64_EmptyArray) {
  uint64_t data[] = {};
  EXPECT_EQ(computeMaxValue64(data, 0), 0);
}

TEST_F(MaxBitsTest, ComputeMaxValue64_SingleElement) {
  uint64_t data[] = {10000000000ULL};
  EXPECT_EQ(computeMaxValue64(data, 1), 10000000000ULL);
}

TEST_F(MaxBitsTest, ComputeMaxValue64_MultipleElements) {
  uint64_t data[] = {1, UINT64_MAX, 1000, 500, 2000};
  EXPECT_EQ(computeMaxValue64(data, 5), UINT64_MAX);
}

// Tests for computeNumBits main function
TEST_F(MaxBitsTest, ComputeNumBits_8bit_AllZeros) {
  uint8_t data[] = {0, 0, 0, 0};
  int result = computeNumBits(data, 4, 1);
  EXPECT_EQ(result, 1); // Should return 1 for all zeros
}

TEST_F(MaxBitsTest, ComputeNumBits_8bit_SingleBit) {
  uint8_t data[] = {1, 1, 1};
  int result = computeNumBits(data, 3, 1);
  EXPECT_EQ(result, 1); // Max value is 1, needs 1 bit
}

TEST_F(MaxBitsTest, ComputeNumBits_8bit_TwoBits) {
  uint8_t data[] = {1, 2, 3};
  int result = computeNumBits(data, 3, 1);
  EXPECT_EQ(result, 2); // Max value is 3, needs 2 bits
}

TEST_F(MaxBitsTest, ComputeNumBits_8bit_EightBits) {
  uint8_t data[] = {1, 100, 255};
  int result = computeNumBits(data, 3, 1);
  EXPECT_EQ(result, 8); // Max value is 255, needs 8 bits
}

TEST_F(MaxBitsTest, ComputeNumBits_16bit_AllZeros) {
  uint16_t data[] = {0, 0, 0, 0};
  int result = computeNumBits(data, 4, 2);
  EXPECT_EQ(result, 1); // Should return 1 for all zeros
}

TEST_F(MaxBitsTest, ComputeNumBits_16bit_MaxValue) {
  uint16_t data[] = {1, 100, 65535};
  int result = computeNumBits(data, 3, 2);
  EXPECT_EQ(result, 16); // Max value is 65535, needs 16 bits
}

TEST_F(MaxBitsTest, ComputeNumBits_32bit_AllZeros) {
  uint32_t data[] = {0, 0, 0, 0};
  int result = computeNumBits(data, 4, 4);
  EXPECT_EQ(result, 1); // Should return 1 for all zeros
}

TEST_F(MaxBitsTest, ComputeNumBits_32bit_MaxValue) {
  uint32_t data[] = {1, 100, UINT32_MAX};
  int result = computeNumBits(data, 3, 4);
  EXPECT_EQ(result, 32); // Max value is UINT32_MAX, needs 32 bits
}

TEST_F(MaxBitsTest, ComputeNumBits_64bit_AllZeros) {
  uint64_t data[] = {0, 0, 0, 0};
  int result = computeNumBits(data, 4, 8);
  EXPECT_EQ(result, 1); // Should return 1 for all zeros
}

TEST_F(MaxBitsTest, ComputeNumBits_64bit_MaxValue) {
  uint64_t data[] = {1, 100, UINT64_MAX};
  int result = computeNumBits(data, 3, 8);
  EXPECT_EQ(result, 64); // Max value is UINT64_MAX, needs 64 bits
}

// Edge cases and boundary tests
TEST_F(MaxBitsTest, ComputeNumBits_EmptyArrays) {
  uint8_t data8[] = {};
  EXPECT_EQ(computeNumBits(data8, 0, 1), 1);

  uint16_t data16[] = {};
  EXPECT_EQ(computeNumBits(data16, 0, 2), 1);

  uint32_t data32[] = {};
  EXPECT_EQ(computeNumBits(data32, 0, 4), 1);

  uint64_t data64[] = {};
  EXPECT_EQ(computeNumBits(data64, 0, 8), 1);
}

TEST_F(MaxBitsTest, ComputeNumBits_PowersOfTwoMinus1) {
  // Test values that are 2^n - 1, which should need exactly n bits
  uint32_t data1[] = {1}; // 2^1 - 1, needs 1 bit
  EXPECT_EQ(computeNumBits(data1, 1, 4), 1);

  uint32_t data3[] = {3}; // 2^2 - 1, needs 2 bits
  EXPECT_EQ(computeNumBits(data3, 1, 4), 2);

  uint32_t data7[] = {7}; // 2^3 - 1, needs 3 bits
  EXPECT_EQ(computeNumBits(data7, 1, 4), 3);

  uint32_t data15[] = {15}; // 2^4 - 1, needs 4 bits
  EXPECT_EQ(computeNumBits(data15, 1, 4), 4);

  uint32_t data255[] = {255}; // 2^8 - 1, needs 8 bits
  EXPECT_EQ(computeNumBits(data255, 1, 4), 8);
}

TEST_F(MaxBitsTest, ComputeNumBits_PowersOfTwo) {
  // Test values that are 2^n, which should need n+1 bits
  uint32_t data1[] = {1}; // 2^0, needs 1 bit
  EXPECT_EQ(computeNumBits(data1, 1, 4), 1);

  uint32_t data2[] = {2}; // 2^1, needs 2 bits
  EXPECT_EQ(computeNumBits(data2, 1, 4), 2);

  uint32_t data4[] = {4}; // 2^2, needs 3 bits
  EXPECT_EQ(computeNumBits(data4, 1, 4), 3);

  uint32_t data8[] = {8}; // 2^3, needs 4 bits
  EXPECT_EQ(computeNumBits(data8, 1, 4), 4);

  uint32_t data256[] = {256}; // 2^8, needs 9 bits
  EXPECT_EQ(computeNumBits(data256, 1, 4), 9);
}

// Test with larger arrays to ensure performance
TEST_F(MaxBitsTest, ComputeNumBits_LargeArray) {
  std::vector<uint32_t> data(10000, 42);
  int result = computeNumBits(data.data(), data.size(), 4);
  EXPECT_EQ(result, 6); // 42 needs 6 bits (42 = 101010b)
}

TEST_F(MaxBitsTest, ComputeNumBits_MixedValues) {
  // Test with a mix of small and large values
  std::vector<uint32_t> data = {1, 2, 4, 8, 16, 32, 64, 128, 256, 512, 1024};
  int result = computeNumBits(data.data(), data.size(), 4);
  EXPECT_EQ(result, 11); // 1024 = 2^10, needs 11 bits
}
