#include "Preprocessor.hpp"

Preprocessor::Preprocessor(std::vector<Tokens::Token> tokens) {
  this->content = tokens;
}

std::vector<Tokens::Token> Preprocessor::preprocess() {
  
  
  return std::vector<Tokens::Token>();
}

void Preprocessor::print(std::ostream &stream) {
  for (Tokens::Token tok : output) {
    tok.print(stream);
  }
}

Tokens::Token Preprocessor::null() {
  return Tokens::nullToken();
}

int Preprocessor::getCurrentLine()
{
  return peek(-1).line;
}

std::string Preprocessor::getCurrentColumn() {
  std::stringstream ss;
  peek(-1).print(ss);
  return ss.str(); 
}

bool Preprocessor::equalCriteria(Tokens::Token a, Tokens::Token b) {
  if (a.type != b.type || (!a.value.empty() && !b.value.empty() && a.value != b.value))
    return false;
  return true;
}
