#pragma once
#include <iostream>
#include <sstream>

namespace StringUtils {
  bool isInString(char c, std::string s);
  std::string parseEscapes(const std::string& input);
}
