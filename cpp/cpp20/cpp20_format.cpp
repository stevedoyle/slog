#include <format>
#include <iostream>

int main() {
  std::cout << std::format("Hello, {}!\n", "world");

  for (auto x : {1, 2, 3}) {
    std::cout << std::format("{}", x);
    std::format_to(std::ostreambuf_iterator<char>(std::cout), " ");
  }
  std::cout << std::endl;
  return 0;
}
