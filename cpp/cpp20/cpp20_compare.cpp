#include <compare>
#include <iostream>

struct foo {
  int a;
  bool b;
  char c;

  // Compare `a` first, then `b`, then `c`, in that order
  auto operator<=>(const foo &) const = default;
};

int main() {
  foo f1{1, true, 'a'};
  foo f2{1, true, 'b'};

  if (f1 < f2) {
    std::cout << "f1 < f2\n";
  } else {
    std::cout << "f1 >= f2\n";
  }
}
