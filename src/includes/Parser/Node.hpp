#pragma once

#include <iostream>
#include <functional>
#include <vector>
#include <any>
#include <Utils/Errors.hpp>
#include <sstream>
#include <variant>
#include <Parser/Literals.hpp>

namespace Node {
  using namespace std;
  class NodeInstance;

  class Property {
    public:
      Property(string name, function<any(NodeInstance&)> criteria);
      Property(function<any(NodeInstance&)> criteria);
      any get();
      string getName();
      bool isValid();
      virtual void invoke(NodeInstance& instance);
    private:
      string name;
      bool validValue = false;
      function<any(NodeInstance&)> criteria;
      any value;
  };

  enum class NodeId {
    scope, func_decl, var_decl, type_decl, public_field, import, namesp, defer, var_set, return_stmt, 
    asm_code, operation_decl, cast_decl, if_stmt, while_stmt, do_while_stmt, for_stmt, alias_decl, alias_use
  };

  std::ostream& operator<<(std::ostream& out, NodeId id);

  class NodeInstance {
    public:
      NodeId id;
      vector<Property*> requirements;
      bool add = true;
      std::function<void(std::ostream&)> onPrint;

      template <typename T>
      T getProperty(string name) {
        for (Property* prop : requirements) {
          if (prop->getName() == name)
            return any_cast<T>(prop->get());
        }
        Errors::error({"Internal Error", "Property not found"});
      }

      void print(std::ostream& stream);
  };

  class Node {
    public:
      Node();
      Node(NodeId name, function<bool()> criteria);
      bool check();
      bool doAdd();
      Node& property(string name, function<any(NodeInstance& instance)> f);
      Node& require(function<void*(NodeInstance& instance)> f);
      Node& finally(function<void(NodeInstance&)> fnal);
      Node& onPrint(function<void(NodeInstance&, std::ostream&)> onprint);
      Node& notAdd();
      void registerNode(vector<Node>& nodes);
      NodeInstance* build();
    private:
      NodeId id;
      vector<Property*> requirements;
      function<bool()> criteria;
      function<void(NodeInstance&)> fnal = [](NodeInstance&){};
      function<void(NodeInstance&, std::ostream&)> on_print = [](NodeInstance& instance, std::ostream& stream){stream << "INSTANCE DOES NOT PROVIDE ON_PRINT";};
      bool doNotAdd = false;
  };
  class Type;
  struct Variable {
    Type* t;
    string name;
  };

  struct Expression;

  struct SubscriptExpr {
    Expression* base;
    Expression* index;
  };

  struct Cast;

  struct CastExpr {
    Expression* expr;
    Cast* cast;
  };

  struct Operation;

  struct CustomExpr {
    Expression* a;
    Expression* b;
    Operation* op;
  };

  enum class ExprType {//!                                                 |                                          |
    literal, variable, func_call, reference, dereference, subscript, dot_notation, cast, interface_ref, custom, interface_call
  };

  struct Expression {
    ExprType type;
    Type* returnType;
    variant<Lits::Literal*, Variable*, NodeInstance*, Expression*, SubscriptExpr*, vector<Variable*>, CastExpr*, CustomExpr*> variant;
  };

  std::ostream& operator<<(std::ostream& stream, const Type& t);
  std::ostream& operator<<(std::ostream& stream, const ExprType etype);
  std::ostream& operator<<(std::ostream& stream, const Expression& e);
  
  class Type {
    public:
      enum class Builtins {
        INT, UINT, LONG, ULONG, FLOAT, DOUBLE, BYTE, CHAR, BOOLEAN, STRING, VOID, STRUCT, UNION, INTERFACE, ALIAS, POINTER
      };

      Type(Builtins type, bool mut = false, string identifier = string(), Type* pointsTo = nullptr, vector<Variable*> fields = {}, vector<Type*> params = {}, Type* retType = nullptr, Expression* expr = nullptr);
      bool operator==(const Type& a) const;
      bool operator!=(const Type& a) const;
      friend std::ostream& operator<<(std::ostream& stream, const Type& t);

      Builtins type;
      bool mut;
      string identifier;
      Type* pointsTo;
      vector<Variable*> fields;
      vector<Type*> params;
      Type* returnType;
      Expression* initSize;
  };

  std::ostream& operator<<(std::ostream& stream, const Type& t);

  struct Operation {
    bool unary;
    string symbols;
    Type* a;
    Type* b;
    Type* r;
    NodeInstance* body;
    int precedence;
    bool operator==(const Operation& a) const;
    bool operator!=(const Operation& a) const;
  };

  struct Cast {
    Type* a;
    Type* b;
    NodeInstance* body;
    bool operator==(const Cast& a) const;
  };
}
