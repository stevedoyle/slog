#include <iostream>
#include <vector>

int main() {
  for (auto v = std::vector{1, 2, 3, 4, 5}; auto i : v) {
    std::cout << i << std::endl;
  }
  return 0;
}
