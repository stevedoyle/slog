#ifndef MAXBITS_H
#define MAXBITS_H

#include <cstddef>
#include <cstdint>

// Main function
int computeNumBits(const void *const src, size_t const numElements,
                   size_t const elementWidth);

// Helper functions (exposed for testing)
uint32_t computeMaxValue8(uint8_t const *src, size_t nbElts);
uint32_t computeMaxValue16(uint16_t const *src, size_t nbElts);
uint32_t computeMaxValue32(uint32_t const *src, size_t nbElts);
uint64_t computeMaxValue64(uint64_t const *src, size_t nbElts);
int highbit64(uint64_t v);

#endif // MAXBITS_H
