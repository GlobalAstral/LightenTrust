#pragma once
#include <iostream>
#include <Utils/Formatting.hpp>

namespace Tokens {
  enum class TokenType {
    null = -1,
    open_paren, close_paren, open_curly, close_curly, open_angle, close_angle, open_square, close_square, semicolon, at, dot, comma, colon, d_colon, pipe, arrow,
    literal, symbols, identifier,
    Var, Int, Uint, Float, Long, Ulong, Double, Char, Byte, Boolean, String, Void, Mutable, Struct, Union, Interface, As,
    Return, Asm, Type, If, Else, While, Do, For, Namespace, Defer, Func, Inline, Public,
    import, public_closure, below, above, all, none, operation, cast, autocast,
    preprocessor, define, ifdef, ifndef, endif, undef, keyword, macro, logi, logw, loge, _template
  };

  struct Token {
    TokenType type;
    unsigned int line;
    std::string value;
    void print(std::ostream& stream);
  };

  Token nullToken();
}
