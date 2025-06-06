#pragma once

#include <Utils/Formatting.hpp>

#define EXTENSION ".lt"

namespace Constants {
  const char LITERAL_LONG = 'L';
  const char LITERAL_FLOAT = 'f';
  const char LITERAL_DOUBLE = 'D';
  const char LITERAL_BINARY = 'b';
  const char LITERAL_OCTAL = 'o';
  const char LITERAL_HEX = 'H';

  const std::string LITERAL_PREFIXES = Formatting::format("%c%c%c%c%c%c", LITERAL_LONG, LITERAL_FLOAT, LITERAL_DOUBLE, LITERAL_BINARY, LITERAL_OCTAL, LITERAL_HEX);
  const std::string NON_SYMBOLS_TOKEN_CHARS = "()[]{}<>;@$:,\"'";
}
