#include <iostream>
#include <span>

void print_ints(std::span<const int> ints) {
  for (const auto i : ints) {
    std::cout << i << ' ';
  }
  std::cout << '\n';
}

int main() {
  print_ints(std::vector{1, 2, 3});
  print_ints(std::array<int, 5>{1, 2, 3, 4, 5});

  int a[10] = {0};
  print_ints(a);

  return 0;
}
