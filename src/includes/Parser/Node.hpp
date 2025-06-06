#pragma once

#include <iostream>
#include <functional>
#include <vector>
#include <any>
#include <Utils/Errors.hpp>

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

  enum class NodeType {
    Expression, Statement
  };

  enum class NodeId {
    scope, func_decl, var_decl, type_decl,
  };

  class NodeInstance {
    public:
      NodeType nodeType;
      NodeId name;
      vector<Property*> requirements;

      template <typename T>
      T getProperty(string name) {
        for (Property* prop : requirements) {
          if (prop->getName() == name)
            return any_cast<T>(prop->get());
        }
        Errors::error({"Internal Error", "Property not found"});
      }
  };

  class Node {
    public:
      Node() { }
      Node(NodeId name, NodeType nodeType, function<bool()> criteria) {
        this->id = name;
        this->nodeType = nodeType;
        this->criteria = criteria;
      }
      bool check() { return criteria(); }

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

      void registerNode(vector<Node>& nodes) {
        nodes.push_back(*this);
      }

      NodeInstance* build() {
        NodeInstance* ret = new NodeInstance();
        ret->name = id;
        ret->nodeType = nodeType;
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
      NodeType nodeType;
      NodeId id;
      vector<Property*> requirements;
      function<bool()> criteria;
      function<void(NodeInstance&)> fnal = [](NodeInstance&){};
  };
  class Type;
  struct Variable {
    Type* t;
    string name;
  };

  class Expression; //TODO

  class Type {
    public:
      enum class Builtins {
        INT, UINT, LONG, ULONG, FLOAT, DOUBLE, BYTE, CHAR, BOOLEAN, STRING, VOID, STRUCT, UNION, INTERFACE, ALIAS, POINTER
      };

      Type(Builtins type, bool mut = false, string identifier = string(), Type* pointsTo = NULL, vector<Variable*> fields = {}) {
        this->type = type;
        this->mut = mut;
        this->identifier = identifier;
        this->pointsTo = pointsTo;
        this->fields = fields;
      }

      bool operator==(const Type& a) const {
        if (this->type != a.type) return false;
        if (mut != a.mut) return false;
        if (!(identifier.empty() && a.identifier.empty() && identifier == a.identifier)) return false;
        if ((pointsTo != NULL && a.pointsTo != NULL ) && (*pointsTo != *(a.pointsTo))) return false;
        if (fields.size() != a.fields.size()) return false;
        for (int i = 0; i < fields.size(); i++) {
          Variable* v1 = fields[i];
          Variable* v2 = a.fields[i];
          if (*(v1->t) != *(v2->t))
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
      Type* pointsTo; //TODO ARRAY LATER
      vector<Variable*> fields;
  };
  

}
