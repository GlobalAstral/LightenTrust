#pragma once

#include <Utils/Processor.hpp>
#include <Tokenizer/Token.hpp>

//TODO DEFINE, IF, UNDEF AND C NORMAL STUFF

/*

$keyword class(cls) struct class_$0 \     - LIKE A NORMAL MACRO BUT NO () NEEDED AND MIGHT HAVE UNIQUE
  NEWLINE EXAMPLE                           FEATURES

class Persona {
  string nome
  string cognome;
  int eta
};

struct class_Persona {
  string nome;
  string cognome;
  int eta;
};

*/

class Preprocessor : public Processor::Processor<Tokens::Token> {
  public:
    Preprocessor(std::vector<Tokens::Token> tokens);
    std::vector<Tokens::Token> preprocess();
    void print(std::ostream& stream);
  private:
    virtual Tokens::Token null();
    virtual int getCurrentLine();
    virtual std::string getCurrentColumn();
    virtual bool equalCriteria(Tokens::Token a, Tokens::Token b);

    std::vector<Tokens::Token> output;
};
