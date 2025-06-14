#include <bit>
#include <iostream>

void endian() {
  if constexpr (std::endian::native == std::endian::big) {
    std::cout << "Big endian\n";
  } else {
    std::cout << "Little endian\n";
  }
}

void bit() {
  unsigned int x = 0b1010'1010'1010'1010;
  std::cout << std::countl_zero(x) << std::endl;
  std::cout << std::countl_one(x) << std::endl;
  std::cout << std::countr_zero(x) << std::endl;
  std::cout << std::countr_one(x) << std::endl;
  std::cout << std::popcount(x) << std::endl;
  x = 2048;
  std::cout << std::has_single_bit(x) << std::endl;
}

int main() {
  endian();
  bit();

  return 0;
}
