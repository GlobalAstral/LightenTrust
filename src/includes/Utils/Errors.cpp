#include <Utils/Errors.hpp>
#include "Errors.hpp"

void Errors::error(std::string type, std::string error, int line) {
  std::string ln = ((line > 0) ? (" AT LINE " + std::to_string(line)) : "");
  std::cout << RED << "ERROR" << ln << ": " << type << " -> " << error << RESET << "\n";
  exit(1);
}

void Errors::error(CompactError error, int line) {
  Errors::error(error.error, error.msg, line);
}
void Errors::warn(std::string warning) {
  std::cout << YELLOW << "WARNING: " << warning << RESET << "\n";
}

void Errors::info(std::string i) {
  std::cout << BLUE << "INFO: " << i << RESET << "\n";
}

#undef RED
#undef YELLOW
#undef BLUE
#undef RESET
