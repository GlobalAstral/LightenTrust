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
      Property(string name, function<any(NodeInstance&)> criteria) {
        this->name = name;
        this->criteria = criteria;
      };

      Property(function<any(NodeInstance&)> criteria) {
        this->name = string();
        this->criteria = criteria;
      };

      any get() {
        if (!validValue)
          Errors::error("Illegal State", "Cannot get value before it is valid");
        return value;
      }

      string getName() {
        return this->name;
      }

      bool isValid() {
        return this->validValue;
      }

      virtual void invoke(NodeInstance& instance) {
        if (validValue)
          Errors::error("Illegal State", "Cannot invoke criteria with valid value");
        this->value = criteria(instance);
        validValue = true;
      }
    private:
      string name;
      bool validValue = false;
      function<any(NodeInstance&)> criteria;
      any value;
  };

  enum class NodeId {
    scope, func_decl, var_decl, type_decl, public_field, import, namesp, defer, var_set, return_stmt, 
    asm_code, operation_decl, cast_decl, if_stmt, while_stmt, do_while_stmt, for_stmt,
  };

  class NodeInstance {
    public:
      NodeId id;
      vector<Property*> requirements;
      bool add = true;

      template <typename T>
      T getProperty(string name) {
        for (Property* prop : requirements) {
          if (prop->getName() == name)
            return any_cast<T>(prop->get());
        }
        Errors::error({"Internal Error", "Property not found"});
      }

      string toString() {
        stringstream ss;
        ss << "NodeInstance(";
        for (Property* prop : requirements)
          ss << prop->getName() << ": " << ((prop->isValid()) ? "T" : "F") << "; ";
        ss << ")";
        return ss.str();
      }
  };

  class Node {
    public:
      Node() { }
      Node(NodeId name, function<bool()> criteria) {
        this->id = name;
        this->criteria = criteria;
      }
      bool check() { return criteria(); }
      bool doAdd() {return !doNotAdd;}

      Node& property(string name, function<any(NodeInstance& instance)> f) {
        Property* prop = new Property(name, f);
        requirements.push_back(prop);
        return *this;
      }

      Node& require(function<void*(NodeInstance& instance)> f) {
        Property* prop = new Property(f);
        requirements.push_back(prop);
        return *this;
      }

      Node& finally(function<void(NodeInstance&)> fnal) {
        this->fnal = fnal;
        return *this;
      }

      Node& notAdd() {
        this->doNotAdd = true;
        return *this;
      }

      void registerNode(vector<Node>& nodes) {
        nodes.push_back(*this);
      }

      NodeInstance* build() {
        NodeInstance* ret = new NodeInstance();
        ret->id = id;
        ret->add = this->doAdd();
        for (Property* prop : requirements) {
          ret->requirements.push_back(new Property(*prop));
        }

        for (Property* prop : ret->requirements) {
          prop->invoke(*ret);
        }
        this->fnal(*ret);
        return ret;
      }

    private:
      NodeId id;
      vector<Property*> requirements;
      function<bool()> criteria;
      function<void(NodeInstance&)> fnal = [](NodeInstance&){};
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

  enum class ExprType {
    literal, variable, func_call, reference, dereference, subscript, dot_notation, cast, interface_ref, custom
  };

  struct Expression {
    ExprType type;
    Type* returnType;
    variant<Lits::Literal*, Variable*, NodeInstance*, Expression*, SubscriptExpr*, vector<Variable*>, CastExpr*, CustomExpr*> variant;
  };

  class Type {
    public:
      enum class Builtins {
        INT, UINT, LONG, ULONG, FLOAT, DOUBLE, BYTE, CHAR, BOOLEAN, STRING, VOID, STRUCT, UNION, INTERFACE, ALIAS, POINTER
      };

      Type(Builtins type, bool mut = false, string identifier = string(), Type* pointsTo = nullptr, vector<Variable*> fields = {}, vector<Type*> params = {}, Type* retType = nullptr, Expression* expr = nullptr) {
        this->type = type;
        this->mut = mut;
        this->identifier = identifier;
        this->pointsTo = pointsTo;
        this->fields = fields;
        this->params = params;
        this->returnType = retType;
        this->initSize = expr;
      }

      bool operator==(const Type& a) const {
        if (this->type != a.type) return false;
        if (mut != a.mut) return false;
        if (!(identifier.empty() && a.identifier.empty() && identifier == a.identifier)) return false;
        if ((pointsTo != nullptr && a.pointsTo != nullptr ) && (*pointsTo != *(a.pointsTo))) return false;
        if (fields.size() != a.fields.size()) return false;
        for (int i = 0; i < fields.size(); i++) {
          Variable* v1 = fields[i];
          Variable* v2 = a.fields[i];
          if (v1->t != nullptr && v2->t != nullptr && *(v1->t) != *(v2->t))
            return false;
        }
        if (this->returnType != nullptr && a.returnType != nullptr && *(this->returnType) != *(a.returnType))
          return false;
        if (this->params.size() != a.params.size())
          return false;

        for (int i = 0; i < params.size(); i++) {
          Type* a_t = params[i];
          Type* b_t = a.params[i];
          if (a_t != nullptr && b_t != nullptr && *a_t != *b_t)
            return false;
        }
        return true;
      }

      bool operator!=(const Type& a) const {
        return !(*this == a);
      }

      Builtins type;
      bool mut;
      string identifier;
      Type* pointsTo;
      vector<Variable*> fields;
      vector<Type*> params;
      Type* returnType;
      Expression* initSize;
  };

  struct Operation {
    bool unary;
    string symbols;
    Type* a;
    Type* b;
    Type* r;
    NodeInstance* body;
    int precedence;
    bool operator==(const Operation& a) const {
      if (unary != a.unary || symbols != a.symbols || (*(this->a) != *(a.a)) || (*(this->b) != *(a.b)) || (*(this->r) != *(a.r))) 
        return false;
      return true;
    }
    bool operator!=(const Operation& a) const {
      return !(*this == a);
    }
  };

  struct Cast {
    Type* a;
    Type* b;
    NodeInstance* body;
    bool operator==(const Cast& a) const {
      return (*(this->a) == *(a.a) && *(this->b) == *(a.b));
    }
  };
}
