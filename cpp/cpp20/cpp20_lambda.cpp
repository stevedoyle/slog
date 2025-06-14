#include <iostream>

struct int_value {
  int n = 0;
  auto getter_fn() {
    return [=, *this]() { return n; };
  }
};

int main() {
  int_value v;
  auto getter = v.getter_fn();
  std::cout << getter() << std::endl;

  v.n = 42;
  std::cout << getter() << std::endl;
  std::cout << v.getter_fn()() << std::endl;

  return 0;
}
