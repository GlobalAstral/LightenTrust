#pragma once

#include <Utils/Processor.hpp>
#include <Tokenizer/Token.hpp>
#include <Utils/Map.hpp>
#include <utility>

namespace Preprocessor {

  struct Template {
    std::vector<std::string> generics;
    std::vector<std::string> params;
    std::vector<Tokens::Token> content;
    std::string body;
  };

  class Preprocessor : public Processor::Processor<Tokens::Token> {
    public:
      Preprocessor(std::vector<Tokens::Token> tokens);
      void print(std::ostream& stream);
      std::vector<Tokens::Token> preprocess();
    protected:
      void preprocessSingle(std::vector<Tokens::Token>& out);
      bool isUnique(std::string name);
      void mustBeUnique(std::string name);
      void mustExist(std::string name);
      std::string getIdentifier();
      void withTokens(std::vector<Tokens::Token>& newTokens, int newPeek, std::function<void(std::vector<Tokens::Token>&, int&)> lambda);
      void withTokens(std::vector<Tokens::Token>& newTokens, std::function<void(std::vector<Tokens::Token>&, int&)> lambda);
      void preprocess(std::vector<Tokens::Token>& out);

      Tokens::Token null();
      int getCurrentLine();
      std::string getCurrentColumn();
      bool equalCriteria(Tokens::Token a, Tokens::Token b);
      std::vector<Tokens::Token>& output = *(new std::vector<Tokens::Token>());

      Map::Map<std::string, std::vector<Tokens::Token>> definitions;
      Map::Map<std::string, std::vector<Tokens::Token>> internal;
      Map::Map<std::string, std::pair<std::vector<std::string>, std::vector<Tokens::Token>>> macros;
      Map::Map<std::string, std::pair<Tokens::Token, std::vector<Tokens::Token>>> keywords;
      Map::Map<std::string, Template> templates;
  };
}
