#pragma once
#include <iostream>
#include <vector>
#include <Tokenizer/Token.hpp>
#include <Utils/Errors.hpp>
#include <Utils/Formatting.hpp>
#include <Utils/StringUtils.hpp>
#include <Utils/Constants.hpp>
#include <Utils/Processor.hpp>

namespace Tokenizer {
  class Tokenizer : public Processor::Processor<char> {
    public:
      Tokenizer(std::string s);
      std::vector<Tokens::Token> tokenize();
    private:
      char null();
      int getCurrentLine();
      bool equalCriteria(char a, char b);

      unsigned int line = 1;
  };
}
