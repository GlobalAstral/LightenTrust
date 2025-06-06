#include <Parser/Parser.hpp>
#include "Parser.hpp"

namespace Parser {

  using namespace Node;

  std::vector<NodeInstance*> Parser::parse() {
    vector<NodeInstance*> ret;
    while (hasPeek()) {
      NodeInstance* node = parseSingle();
      ret.push_back(node);
    }
    return ret;
  }

  void Parser::registerNodes() {
    Node::Node{NodeId::scope, NodeType::Statement, [this](){return tryconsume({Tokens::TokenType::open_curly});}}
    .property("content", [this](Node::NodeInstance& instance){
      vector<Node::NodeInstance*> buf;
      int v_index = vars.size();
      this->scopeHierarchy++;
      if (!doUntilFind({Tokens::TokenType::close_curly}, [&buf, this](){
        buf.push_back(parseSingle());
      })) error({"Missing Token", "Expected '}'"});
      if (v_index >= 0)
        vars.erase(vars.begin()+v_index);
      this->scopeHierarchy--;
      return buf;
    }).require([this](Node::NodeInstance& instance){ tryconsume({Tokens::TokenType::semicolon}, {"Missing Token", "Expected ';'"}); return (void*)0; })
    .registerNode(this->nodes);

    Node::Node{NodeId::func_decl, NodeType::Statement, [this](){ return tryconsume({Tokens::TokenType::Func}); }}
    .property("public", [this](NodeInstance& instance){ return tryconsume({Tokens::TokenType::Public}); })
    .property("inline", [this](NodeInstance& instance){ return tryconsume({Tokens::TokenType::Inline}); })
    .property("name", [this](NodeInstance& instance){ return tryconsume({Tokens::TokenType::identifier}, {"Missing Token", "Expected Identifier"}).value; })
    .property("parameters", [this](NodeInstance& instance){
      tryconsume({Tokens::TokenType::open_paren}, {"Missing Token", "Expected '('"});
      vector<Variable*> params;
      if (!doUntilFind({Tokens::TokenType::close_paren}, [this, &params](){
        Variable* var = parseVar();
        if (varExists(var, this->vars)) 
          error({"Redefinition Error", "Variable already exists"});
        if (varExists(var, params)) error({"Redefinition Error", "Parameter already exists"});
        params.push_back(var);
      }, {Tokens::TokenType::comma}, {"Expected separating comma"})) error({"Missing Token", "Expected ')'"});
      return params;
    }).property("returnType", [this](NodeInstance& instance){tryconsume({Tokens::TokenType::colon}, {"Missing Token", "Expected return type specifier"}); return parseType(); })
    .property("body", [this](NodeInstance& instance){
      if (tryconsume({Tokens::TokenType::semicolon}))
        return (NodeInstance*)NULL;
      int index = this->vars.size();
      vector<Variable*> params = instance.getProperty<vector<Variable*>>("parameters");
      for (Variable* var : params)
        this->vars.push_back(var);
      NodeInstance* body = parseSingle();
      if (body->id != NodeId::scope)
        error({"Syntax Error", "Scope Expected"});
      if (index >= 0)
        this->vars.erase(this->vars.begin()+index);
      return body;
    }).finally([this](NodeInstance& instance) {
      if (this->scopeHierarchy > 0)
        error({"Logic Error", "Cannot declare a function inside a scope"});
      if (funcHasBody(&instance, this->functions))
        error({"Redefinition Error", "Function already exists"});
      this->functions.push_back(&instance);
    }).registerNode(this->nodes);

    Node::Node(NodeId::var_decl, NodeType::Statement, [this](){ return tryconsume({Tokens::TokenType::Var}); })
    .property("name", [this](NodeInstance& instance){ return tryconsume({Tokens::TokenType::identifier}, {"Missing Token", "Expected Identifier"}).value; })
    .property("type", [this](NodeInstance& instance){ tryconsume({Tokens::TokenType::colon}, {"Missing Token", "Expected type specifier"}); return parseType(); })
    .property("value", [this](NodeInstance& instance){
      if (tryconsume({.type=Tokens::TokenType::symbols, .value="="}))
        return parseExpr();
      return (Node::Expression*)NULL;
    }).require([this](NodeInstance& instance){tryconsume({Tokens::TokenType::semicolon}, {"Missing Token", "Expected ';'"}); return (void*)0;})
    .finally([this](NodeInstance& instance){
      string name = instance.getProperty<string>("name");
      Type* t = instance.getProperty<Type*>("type");
      Variable* var = new Variable{t, name};
      if (varExists(var, this->vars))
        error({"Redefinition Error", "Variable already exists"});
      this->vars.push_back(var);
    }).registerNode(this->nodes);

    Node::Node{NodeId::type_decl, NodeType::Statement, [this](){ return tryconsume({Tokens::TokenType::Type}); }}
    .property("alias", [this](NodeInstance& instance){ return tryconsume({Tokens::TokenType::identifier}, {"Missing Token", "Expected Identifier"}).value; })
    .property("type", [this](NodeInstance& instance) { return tryconsume({Tokens::TokenType::semicolon}) ? NULL : parseType(); })
    .finally([this](NodeInstance& instance) {
      string alias = instance.getProperty<string>("alias");
      Type* t = instance.getProperty<Type*>("type");
      if (!declaredTypes.contains(alias) || (declaredTypes.contains(alias) && declaredTypes[alias] == NULL)) {
        declaredTypes[alias] = t;
      } else { error({"Redefinition Error", "Cannot declare already existing type"}); }
    }).registerNode(this->nodes);
  }

  NodeInstance* Parser::parseSingle() {
    for (Node::Node node : this->nodes) {
      if (node.check()) {
        NodeInstance* ret = node.build();
        return ret;
      }
    }
    error({"Syntax Error", "Invalid Statement"});
  }

  Node::Type *Parser::parseType() {
    bool mut = tryconsume({Tokens::TokenType::Mutable});
    if (tryconsume({.type=Tokens::TokenType::symbols, .value="&"}))
      return new Type(Type::Builtins::POINTER, mut, string(), parseType());
    if (tryconsume({Tokens::TokenType::Int})) {
      return new Type(Type::Builtins::INT, mut);
    } else if (tryconsume({Tokens::TokenType::Uint})) {
      return new Type(Type::Builtins::UINT, mut);
    } else if (tryconsume({Tokens::TokenType::Long})) {
      return new Type(Type::Builtins::LONG, mut);
    } else if (tryconsume({Tokens::TokenType::Ulong})) {
      return new Type(Type::Builtins::ULONG, mut);
    } else if (tryconsume({Tokens::TokenType::Float})) {
      return new Type(Type::Builtins::FLOAT, mut);
    } else if (tryconsume({Tokens::TokenType::Double})) {
      return new Type(Type::Builtins::DOUBLE, mut);
    } else if (tryconsume({Tokens::TokenType::Char})) {
      return new Type(Type::Builtins::CHAR, mut);
    } else if (tryconsume({Tokens::TokenType::Byte})) {
      return new Type(Type::Builtins::BYTE, mut);
    } else if (tryconsume({Tokens::TokenType::Boolean})) {
      return new Type(Type::Builtins::BOOLEAN, mut);
    } else if (tryconsume({Tokens::TokenType::String})) {
      return new Type(Type::Builtins::STRING, mut);
    } else if (tryconsume({Tokens::TokenType::Void})) {
      return new Type(Type::Builtins::VOID, mut);
    } else if (peek().type == Tokens::TokenType::Struct || peek().type == Tokens::TokenType::Union) {
      Type::Builtins t = (tryconsume({Tokens::TokenType::Struct})) ? Type::Builtins::STRUCT : Type::Builtins::UNION;
      if (tryconsume({Tokens::TokenType::semicolon}))
        return new Type(t);
      tryconsume({Tokens::TokenType::open_curly}, {"Missing Token", "Expected '{'"});
      Type* ret = new Type(t);
      bool found = doUntilFind({Tokens::TokenType::close_curly}, [this, &ret](){
        Variable* var = parseVar();
        ret->fields.push_back(var);
        tryconsume({Tokens::TokenType::semicolon}, {"Missing Token", "Expected ';'"});
      });
      if (!found) error({"Missing Token", "Expected '}'"});
      ret->mut = mut;
      return ret;
    } else if (tryconsume({Tokens::TokenType::Interface})) {
      return new Type(Type::Builtins::INTERFACE, mut);
    } else if (peek().type == Tokens::TokenType::identifier) {
      string name = consume().value;
      if (!declaredTypes.contains(name))
        error({"Syntax Error", "Invalid Type"});
      Type* declType = declaredTypes[name];
      if (declType == NULL)
        return new Type(Type::Builtins::POINTER, true, string(), new Type(Type::Builtins::VOID));
      Type* t = new Type(declType->type, declType->mut, declType->identifier, declType->pointsTo, declType->fields);
      t->mut = t->mut || mut;
      return t;
    }
    error({"Syntax Error", "Invalid Type"});
  }
  Node::Variable* Parser::parseVar() {
    string name = tryconsume({Tokens::TokenType::identifier}, {"Missing Token", "Expected identifier"}).value;
    tryconsume({Tokens::TokenType::colon}, {"Missing Token", "Expected type specifier"});
    Type* t = parseType();
    return new Variable{t, name};
  }

  Node::Expression *Parser::parseExpr() {
    return nullptr;
  }

  bool Parser::funcHasBody(Node::NodeInstance* instance, vector<Node::NodeInstance*> &funcs) {
    if (funcs.empty()) 
      return false;
    for (int i = 0; i < funcs.size(); i++) {
      NodeInstance* func = funcs[i];
      if (func->getProperty<string>("name") != instance->getProperty<string>("name"))
        continue;
      Type* aT = func->getProperty<Type*>("returnType");
      Type* bT = instance->getProperty<Type*>("returnType");
      if (*aT != *bT)
        continue;
      vector<Variable*> a = func->getProperty<vector<Variable*>>("parameters");
      vector<Variable*> b = instance->getProperty<vector<Variable*>>("parameters");
      if (a.size() != b.size()) continue;
      bool sameParams = true;
      for (int i = 0; i < a.size(); i++) {
        if (*(a[i]->t) != *(b[i]->t)) {
          sameParams = false;
          break;
        }
      }
      if (!sameParams)
        continue;
      if (func->getProperty<NodeInstance*>("body") != nullptr) {
        return true;
      }
      funcs.erase(funcs.begin()+i, funcs.begin()+i+1);
    }
    return false;
  }
  bool Parser::varExists(Node::Variable *var, vector<Node::Variable *> &variables) {
    for (Variable* v : variables) {
      if (var->name == v->name)
        return true;
    }
    return false;
  }
}
