#include "maxbits.h"
#include <algorithm>
#include <cassert>
#include <cstddef>
#include <cstdint>

uint32_t computeMaxValue8(uint8_t const *src, size_t nbElts) {
  uint32_t max = 0;
  for (size_t i = 0; i < nbElts; ++i) {
    max = std::max(max, (uint32_t)src[i]);
  }
  return max;
}

uint32_t computeMaxValue16(uint16_t const *src, size_t nbElts) {
  uint32_t max = 0;
  for (size_t i = 0; i < nbElts; ++i) {
    max = std::max(max, (uint32_t)src[i]);
  }
  return max;
}

uint32_t computeMaxValue32(uint32_t const *src, size_t nbElts) {
  uint32_t max = 0;
  for (size_t i = 0; i < nbElts; ++i) {
    max = std::max(max, (uint32_t)src[i]);
  }
  return max;
}

uint64_t computeMaxValue64(uint64_t const *src, size_t nbElts) {
  uint64_t max = 0;
  for (size_t i = 0; i < nbElts; ++i) {
    max = std::max(max, (uint64_t)src[i]);
  }
  return max;
}

int highbit64(uint64_t v) {
  if (v == 0)
    return -1;
  int r = 0;
  while (v != 0) {
    v >>= 1;
    r++;
  }
  return r - 1;
}

int computeNumBits(const void *const src, size_t const numElements,
                   size_t const elementWidth) {
  uint64_t maxValue;
  switch (elementWidth) {
  default:
    assert(false);
  case 1:
    maxValue = computeMaxValue8((uint8_t const *)src, numElements);
    break;
  case 2:
    maxValue = computeMaxValue16((uint16_t const *)src, numElements);
    break;
  case 4:
    maxValue = computeMaxValue32((uint32_t const *)src, numElements);
    break;
  case 8:
    maxValue = computeMaxValue64((uint64_t const *)src, numElements);
    break;
  }
  // Wastes bits when maxValue == 0...
  return 1 + (maxValue == 0 ? 0 : (int)highbit64(maxValue));
}
