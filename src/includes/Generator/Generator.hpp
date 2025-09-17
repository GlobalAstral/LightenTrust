#pragma once

#include <Utils/Processor.hpp>
#include <Tokenizer/Token.hpp>
#include <sstream>
#include <Parser/Node.hpp>
#include <Utils/Map.hpp>
#include <vector>
#include <Generator/Registers.hpp>

namespace Generator {
  typedef long long Long;
  using namespace std;
  using namespace Node;

  struct AssemblyValue {
    enum {
      CONSTANT,
      REGISTER,
      OFFSET,
      ADDRESS,
      LABEL,
    } type;
    union {
      Lits::Literal* constant;
      Long stackOffset;
      AssemblyValue* memoryAddress;
      Register* reg;
      char* label;
    } u;
  };

  class Generator : public Processor::Processor<Node::NodeInstance*> {
    public:
      Generator(vector<Node::NodeInstance*>& nodes, vector<Node::Variable*>& vars, vector<Node::NodeInstance*>& functions, Map::Map<string, vector<Node::NodeInstance*>>& aliases, vector<Node::Operation>& operators, vector<Node::Cast>& casts, vector<Node::Cast>& autocasts, Map::Map<string, Node::Type*>& declaredTypes) {
        this->content = nodes;
        this->vars = &vars;
        this->functions = &functions;
        this->aliases = &aliases;
        this->operators = &operators;
        this->casts = &casts;
        this->autocasts = &autocasts;
        this->declaredTypes = &declaredTypes;

        this->output.str("global main\n");
        this->sec_bss.str("section .bss\n");
        this->sec_data.str("section .data\n");
        this->labels.str("section .text\n");
        this->sec_text.str("section .text\n");
      };
      void print(std::ostream& stream);

      std::string getOutput();

    protected:
      virtual Node::NodeInstance* null() { return new Node::NodeInstance(); }
      virtual int getCurrentLine() { return -1; };
      virtual string getCurrentColumn() { 
        std::stringstream ss;
        peek(-1)->print(ss);
        return ss.str(); 
      };
      virtual bool equalCriteria(Node::NodeInstance* a, Node::NodeInstance* b) {
        if (a->id != b->id)
          return false;
        return true;
      };
    private:
      private:
        Long getSizeof(Type*);

      std::stringstream output;

      std::stringstream sec_text;
      std::stringstream sec_data;
      std::stringstream sec_bss;
      std::stringstream labels;
      
      vector<Node::Variable*>* vars;
      vector<Node::NodeInstance*>* functions;
      Map::Map<string, vector<Node::NodeInstance*>>* aliases;
      vector<Node::Operation>* operators;
      vector<Node::Cast>* casts;
      vector<Node::Cast>* autocasts;
      Map::Map<string, Node::Type*>* declaredTypes;
  };
}
