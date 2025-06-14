#include <iostream>
#include <string>

enum class Color { RED, GREEN, BLUE };

std::string_view to_string(Color color) {
  switch (color) {
    using enum Color;
  case RED:
    return "Red";
  case GREEN:
    return "Green";
  case BLUE:
    return "Blue";
  }
  return "Unknown";
}

int main() {
  Color color = Color::RED;
  std::cout << "Color: " << to_string(color) << std::endl;
  return 0;
}
