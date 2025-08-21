#pragma once
#include <iostream>
#include <vector>
#include <Tokenizer/Token.hpp>
#include <Utils/Errors.hpp>
#include <Utils/Formatting.hpp>
#include <Utils/StringUtils.hpp>
#include <Utils/Constants.hpp>
#include <Utils/Processor.hpp>
#include <sstream>

namespace Tokenizer {
  class Tokenizer : public Processor::Processor<char> {
    public:
      Tokenizer(std::string s);
      std::vector<Tokens::Token> tokenize();
      void print(std::ostream& stream);
    private:
      char null();
      int getCurrentLine();
      std::string getCurrentColumn();
      bool equalCriteria(char a, char b);

      unsigned int line = 1;
      std::vector<Tokens::Token>& output = *(new std::vector<Tokens::Token>());
  };
}
