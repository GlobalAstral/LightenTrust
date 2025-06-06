#include <Utils/StringUtils.hpp>

bool StringUtils::isInString(char c, std::string s) {
  for (char ch : s) {
    if (c == ch)
      return true;
  }
  return false;
}

#include <cctype>

char hexToChar(char h1, char h2) {
  auto hex = [](char c) -> int {
    if (isdigit(c)) return c - '0';
    return std::tolower(c) - 'a' + 10;
  };
  return static_cast<char>((hex(h1) << 4) | hex(h2));
}

std::string StringUtils::parseEscapes(const std::string &input) {
  std::stringstream result;
  size_t i = 0;
  while (i < input.length()) {
    if (input[i] == '\\' && i + 1 < input.length()) {
      char next = input[i + 1];
      switch (next) {
        case 'n': result << '\n'; break;
        case 't': result << '\t'; break;
        case 'r': result << '\r'; break;
        case '\\': result << '\\'; break;
        case '\'': result << '\''; break;
        case '\"': result << '\"'; break;
        case 'a': result << '\a'; break;
        case 'b': result << '\b'; break;
        case 'f': result << '\f'; break;
        case 'v': result << '\v'; break;
        case '0': result << '\0'; break;
        case 'x': {
          if (i + 3 < input.length() && std::isxdigit(input[i + 2]) && std::isxdigit(input[i + 3])) {
            result << hexToChar(input[i + 2], input[i + 3]);
            i += 4;
          } else {
            result << '\\' << 'x';
            i += 2;
          }
          break; 
        }
        default:
          result << '\\' << next;
          break;
      }
      i += 2;
    } else {
      result << input[i];
      i++;
    }
  }
  std::string ret = result.str();
  return ret;
}
