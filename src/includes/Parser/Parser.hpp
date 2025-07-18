#pragma once

#include <Parser/Node.hpp>
#include <Parser/Literals.hpp>
#include <Tokenizer/Token.hpp>
#include <Utils/Processor.hpp>
#include <Utils/Map.hpp>
#include <Utils/Constants.hpp>
#include <fstream>
#include <Tokenizer/Tokenizer.hpp>
#include <Utils/VectorUtils.hpp>
#include <limits>

namespace Parser {
  using namespace std;
  class Parser : Processor::Processor<Tokens::Token> {
    public:
      Parser(vector<Tokens::Token>& tokens) {
        this->content = tokens;
        registerNodes(this->output);
      };

      vector<Node::NodeInstance*> parse();

    protected:
      virtual Tokens::Token null() { return Tokens::nullToken(); };
      virtual int getCurrentLine() { return peek(-1).line; };
      virtual bool equalCriteria(Tokens::Token a, Tokens::Token b) {
        if (a.type != b.type || (!a.value.empty() && !b.value.empty() && a.value != b.value))
          return false;
        return true;
      };

    private:
      void registerNodes(vector<Node::NodeInstance*>& output);
      Node::NodeInstance* parseSingle();

      Node::Type* parseType();
      Node::Variable* parseVar();
      Node::Expression* parseExpr(Node::Type* requiredType);
      vector<Tokens::Token> parseFile(string path, string fieldName);
      Tokens::Token getIdentifier();
      Tokens::Token decodeIdentifier();

      bool funcHasBody(Node::NodeInstance* instance, vector<Node::NodeInstance*>& funcs);
      bool varExists(Node::Variable* var, vector<Node::Variable*>& variables);
      Node::Variable* getVar(Node::Variable* var, vector<Node::Variable*>& variables);
      int findOperation(Node::Operation op, vector<Node::Operation>& operations);
      int findCast(Node::Cast cast, vector<Node::Cast>& casts);
      bool literalIsType(Lits::Literal* lit, Node::Type* t);
      Node::Type* literalType(Lits::Literal* lit);

      vector<Node::Node> nodes;
      vector<Node::Variable*> vars;
      vector<Node::NodeInstance*> functions;
      vector<Node::NodeInstance*> output;
      vector<string> namespaces;
      vector<Node::NodeInstance*> defers;
      vector<Node::Operation> operators;
      vector<Node::Cast> casts;
      vector<Node::Cast> autocasts;
      Map::Map<string, Node::Type*> declaredTypes;
      int scopeHierarchy = 0;
  };
}
