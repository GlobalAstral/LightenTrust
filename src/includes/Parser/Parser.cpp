#include <Parser/Parser.hpp>
#include "Parser.hpp"

namespace Parser {

  using namespace Node;

  std::vector<NodeInstance*> Parser::parse() {
    while (hasPeek()) {
      NodeInstance* node = parseSingle();
      if (node->add)
        this->output.push_back(node);
    }
    return this->output;
  }

  void Parser::registerNodes(vector<NodeInstance*>& output){
    Node::Node{NodeId::scope, [this](){return tryconsume({Tokens::TokenType::open_curly});}}
    .property("content", [this](Node::NodeInstance& instance){
      vector<Node::NodeInstance*> buf;
      int v_index = vars.size();
      this->scopeHierarchy++;
      if (!doUntilFind({Tokens::TokenType::close_curly}, [&buf, this](){
        NodeInstance* node = parseSingle();
        if (node->add)
          buf.push_back(node);
      })) error({"Missing Token", "Expected '}'"});
      for (int i = defers.size()-1; i >= 0; i--) {
        if (defers[i]->add)
          buf.push_back(defers[i]);
      }
      if (v_index >= 0)
        vars.erase(vars.begin()+v_index);
      this->scopeHierarchy--;
      return buf;
    }).require([this](Node::NodeInstance& instance){ tryconsume({Tokens::TokenType::semicolon}, {"Missing Token", "Expected ';'"}); return (void*)0; })
    .registerNode(this->nodes);

    Node::Node{NodeId::func_decl, [this](){ return tryconsume({Tokens::TokenType::Func}); }}
    .property("inline", [this](NodeInstance& instance){ return tryconsume({Tokens::TokenType::Inline}); })
    .property("name", [this](NodeInstance& instance){ return getIdentifier().value; })
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

    Node::Node(NodeId::var_decl, [this](){ return tryconsume({Tokens::TokenType::Var}); })
    .property("name", [this](NodeInstance& instance){ return getIdentifier().value; })
    .property("type", [this](NodeInstance& instance){ tryconsume({Tokens::TokenType::colon}, {"Missing Token", "Expected type specifier"}); return parseType(); })
    .property("value", [this](NodeInstance& instance){
      Type* type = instance.getProperty<Type*>("type");
      if (tryconsume({.type=Tokens::TokenType::symbols, .value="="})) {
        return parseExpr(type);
      }
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

    Node::Node{NodeId::type_decl, [this](){ return tryconsume({Tokens::TokenType::Type}); }}
    .property("alias", [this](NodeInstance& instance){ return getIdentifier().value; })
    .property("type", [this](NodeInstance& instance) { 
      if (tryconsume({Tokens::TokenType::semicolon})) 
        return (Type*)NULL; 
      Type* t = parseType();
      tryconsume({Tokens::TokenType::semicolon}, {"Missing Token", "Expected ';'"});
      return t;
    }).finally([this](NodeInstance& instance) {
      string alias = instance.getProperty<string>("alias");
      Type* t = instance.getProperty<Type*>("type");
      if (!declaredTypes.contains(alias) || (declaredTypes.contains(alias) && declaredTypes[alias] == NULL)) {
        declaredTypes[alias] = t;
      } else { error({"Redefinition Error", "Cannot declare already existing type"}); }
    }).registerNode(this->nodes);

    Node::Node{NodeId::public_field, [this](){ return tryconsume({Tokens::TokenType::Public}); }}
    .property("name", [this](NodeInstance& instance){ return tryconsume({Tokens::TokenType::identifier}, {"Missing Token", "Expected Identifier"}).value; })
    .property("content", [this](NodeInstance& instance){
      tryconsume({Tokens::TokenType::public_closure}, {"Missing Token '$'"});
      vector<NodeInstance*> content;
      if (!doUntilFind({Tokens::TokenType::public_closure}, [this, &content](){
        content.push_back(parseSingle());
      })) error({"Missing Token", "Expected '$'"});
      return content;
    }).registerNode(this->nodes);

    Node::Node{NodeId::import, [this](){ return tryconsume({Tokens::TokenType::import}); }}
    .property("path", [this](NodeInstance& instance){
      vector<string> path;
      if (!doUntilFind({Tokens::TokenType::semicolon}, [this, &path](){
        path.push_back(tryconsume({Tokens::TokenType::identifier}, {"Missing Token", "Expected Identifier"}).value);
      }, {Tokens::TokenType::dot}, {"Missing Token", "Expected '.' separator"}))
        error({"Missing Token", "Expected ';'"});
      return path;
    }).finally([this](NodeInstance& instance){
      vector<string> path = instance.getProperty<vector<string>>("path");
      if (path.size() < 2) error({"File Error", "Invalid path for import statement"});
      string fieldName = path.back();
      string fileName = *(path.end()-2);
      stringstream filePath;
      for (auto i = path.begin(); i != (path.end()-2); ++i)
        filePath << *i << "\\";
      filePath << fileName << EXTENSION;
      string fPath = filePath.str();
      vector<Tokens::Token> imported = parseFile(fPath, fieldName);
      for (int i = imported.size()-1; i >= 0; i--)
        this->content.insert(this->content.begin()+this->_peek, imported[i]);
    }).notAdd().registerNode(this->nodes);

    Node::Node{NodeId::namesp, [this](){ return tryconsume({Tokens::TokenType::Namespace}); }}
    .notAdd().property("name", [this](NodeInstance& instance){return tryconsume({Tokens::TokenType::identifier}, {"Missing Token", "Expected identifier"}).value;})
    .finally([this, &output](NodeInstance& instance){
      tryconsume({Tokens::TokenType::open_curly}, {"Missing Token", "Expected '{'"});
      string name = instance.getProperty<string>("name");
      if (VectorUtils::find<string>(this->namespaces, name) != -1)
        error({"Logic Error", "Namespace already in use"});
      this->namespaces.push_back(name);
      if (!doUntilFind({Tokens::TokenType::close_curly}, [this, &output](){
        NodeInstance* node = parseSingle();
        if (node->add)
          output.push_back(node);
      })) error({"Missing Token", "Expected '}'"});
    }).registerNode(this->nodes);

    Node::Node{NodeId::defer, [this](){ return tryconsume({Tokens::TokenType::Defer}); }}
    .finally([this](NodeInstance& instance){
      if (this->scopeHierarchy <= 0)
        error({"Logic Error", "Cannot use defer out of scope"});
      NodeInstance* node = parseSingle();
      defers.push_back(node);
    }).notAdd().registerNode(this->nodes);

    Node::Node(NodeId::var_set, [this](){ return peek().type == Tokens::TokenType::identifier; })
    .property("name", [this](NodeInstance& instance){ return getIdentifier().value; })
    .property("value", [this](NodeInstance& instance){
      string name = instance.getProperty<string>("name");
      Variable* var = getVar(new Variable{NULL, name}, this->vars);
      if (tryconsume({.type=Tokens::TokenType::symbols, .value="="})) {
        return parseExpr(var->t);
      }
      error({"Missing Token", "Expected '='"});
    }).require([this](NodeInstance& instance){tryconsume({Tokens::TokenType::semicolon}, {"Missing Token", "Expected ';'"}); return (void*)0;})
    .registerNode(this->nodes);

    Node::Node(NodeId::return_stmt, [this](){ return tryconsume({Tokens::TokenType::Return}); })
    .property("value", [this](NodeInstance& instance){ return parseExpr(NULL); })
    .require([this](NodeInstance& instance){tryconsume({Tokens::TokenType::semicolon}, {"Missing Token", "Expected ';'"}); return nullptr;})
    .registerNode(this->nodes);
    
    Node::Node(NodeId::asm_code, [this](){ return peek().type == Tokens::TokenType::Asm; })
    .property("code", [this](NodeInstance& instance){ return consume().value; })
    .require([this](NodeInstance& instance){tryconsume({Tokens::TokenType::semicolon}, {"Missing Token", "Expected ';'"}); return nullptr;})
    .registerNode(this->nodes);

    Node::Node(NodeId::operation_decl, [this](){ return tryconsume({Tokens::TokenType::operation}); })
    .property("symbol", [this](NodeInstance& instance){ return tryconsume({Tokens::TokenType::symbols}, {"Missing Token", "Expected symbols"}).value; })
    .require([this](NodeInstance& instance){tryconsume({Tokens::TokenType::open_angle}, {"Missing Token", "Expected '<'"}); return nullptr;})
    .property("operand1", [this](NodeInstance& instance){ return parseVar(); })
    .require([this](NodeInstance& instance){tryconsume({Tokens::TokenType::comma}, {"Missing Token", "Expected ','"}); return nullptr;})
    .property("operand2", [this](NodeInstance& instance){
      if (peek().type == Tokens::TokenType::below || peek().type == Tokens::TokenType::above || peek().type == Tokens::TokenType::none)
        return (Node::Variable*) nullptr;
      Variable* var = parseVar();
      tryconsume({Tokens::TokenType::comma}, {"Missing Token", "Expected ','"});
      return var;
    }).property("precedence", [this](NodeInstance& instance){
      if (peek().type != Tokens::TokenType::below && peek().type != Tokens::TokenType::above && peek().type != Tokens::TokenType::none)
        error({"Syntax Error", "Expected precedence specifier"});
      if (tryconsume({Tokens::TokenType::none}))
        return 0;
      Tokens::Token clause = consume(); // ABOVE OR BELOW
      if (tryconsume({Tokens::TokenType::all})) {
        if (clause.type == Tokens::TokenType::above)
          return numeric_limits<int>::max();
        return numeric_limits<int>::min();
      }
      Operation tofind;
      tofind.unary = false;
      if (peek().type == Tokens::TokenType::symbols) {
        tofind.a = nullptr;
        tofind.unary = true;
      } else {
        tofind.a = parseType();
      }
      tofind.symbols = tryconsume({Tokens::TokenType::symbols}, {"Missing Token", "Expected symbols"}).value;
      if (tofind.unary)
        tofind.a = parseType();
      else
        tofind.b = parseType();
      tryconsume({Tokens::TokenType::pipe}, {"Missing Token", "Expected '|'"});
      tofind.r = parseType();
      int index = findOperation(tofind, this->operators);
      if (index < 0) error({"Syntax Error", "Operation does not exist"});
      int basePrec = this->operators.at(index).precedence;
      if (clause.type == Tokens::TokenType::above)
        return basePrec+1;
      return basePrec-1;
    }).require([this](NodeInstance& instance){tryconsume({Tokens::TokenType::close_angle}, {"Missing Token", "Expected '>'"}); return nullptr;})
    .property("retType", [this](NodeInstance& instance){ return parseType(); })
    .property("body", [this](NodeInstance& instance){
      int prev = vars.size();
      Variable* var1 = instance.getProperty<Variable*>("operand1");
      Variable* var2 = instance.getProperty<Variable*>("operand2");
      vars.push_back(var1);
      if (var2 != nullptr) vars.push_back(var2);
      NodeInstance* body = parseSingle();
      if (prev >= 0)
        vars.erase(vars.begin()+prev);
      return body;
    }).notAdd().finally([this](NodeInstance& instance){
      Variable* var1 = instance.getProperty<Variable*>("operand1");
      Variable* var2 = instance.getProperty<Variable*>("operand2");
      string symbols = instance.getProperty<string>("symbol");
      Type* retType = instance.getProperty<Type*>("retType");
      NodeInstance* body = instance.getProperty<NodeInstance*>("body");
      Operation op{var2 == nullptr, symbols, var1->t, ((var2 == nullptr) ? nullptr : var2->t), retType, body};
      if (findOperation(op, this->operators) > -1)
        error({"Syntax Error", "Operation already exists"});
      this->operators.push_back(op);
    }).registerNode(this->nodes);

    Node::Node(NodeId::cast_decl, [this](){ return peek().type == Tokens::TokenType::cast || peek().type == Tokens::TokenType::autocast; })
    .property("auto", [this](NodeInstance& instance){ return consume().type == Tokens::TokenType::autocast; })
    .require([this](NodeInstance& instance){ tryconsume({Tokens::TokenType::open_angle}, {"Missing Token", "Expected '<'"}); return nullptr; })
    .property("operand", [this](NodeInstance& instance){ return parseVar(); })
    .require([this](NodeInstance& instance){ tryconsume({Tokens::TokenType::close_angle}, {"Missing Token", "Expected '>'"}); return nullptr; })
    .property("retType", [this](NodeInstance& instance){ return parseType(); })
    .property("body", [this](NodeInstance& instance){ return parseSingle(); })
    .notAdd().finally([this](NodeInstance& instance){
      bool autocast = instance.getProperty<bool>("auto");
      Variable* var = instance.getProperty<Variable*>("operand");
      Type* retType = instance.getProperty<Type*>("retType");
      NodeInstance* body = instance.getProperty<NodeInstance*>("body");
      vector<Cast>& list = this->casts;
      if (autocast) { list = this->autocasts; }
      Cast cast{var->t, retType, body};
      if (findCast(cast, list) > -1)
        error({"Syntax Error", "Cast already exists"});
      list.push_back(cast);
    }).registerNode(this->nodes);

    Node::Node(NodeId::if_stmt, [this](){ return tryconsume({Tokens::TokenType::If}); })
    .require([this](NodeInstance& instance){tryconsume({Tokens::TokenType::open_paren}, {"Missing Token", "Expected '('"}); return nullptr;})
    .property("expr", [this](NodeInstance& instance){ return parseExpr(new Type{Type::Builtins::BOOLEAN}); })
    .require([this](NodeInstance& instance){tryconsume({Tokens::TokenType::close_paren}, {"Missing Token", "Expected ')'"}); return nullptr;})
    .property("body", [this](NodeInstance& instance){ return parseSingle(); })
    .property("else", [this](NodeInstance& instance){ return ((tryconsume({Tokens::TokenType::Else})) ? parseSingle() : nullptr); })
    .registerNode(this->nodes);

    Node::Node(NodeId::while_stmt, [this](){ return tryconsume({Tokens::TokenType::While}); })
    .require([this](NodeInstance& instance){tryconsume({Tokens::TokenType::open_paren}, {"Missing Token", "Expected '('"}); return nullptr;})
    .property("expr", [this](NodeInstance& instance){ return parseExpr(new Type{Type::Builtins::BOOLEAN}); })
    .require([this](NodeInstance& instance){tryconsume({Tokens::TokenType::close_paren}, {"Missing Token", "Expected ')'"}); return nullptr;})
    .property("body", [this](NodeInstance& instance){ return parseSingle(); })
    .registerNode(this->nodes);

    Node::Node(NodeId::do_while_stmt, [this](){ return tryconsume({Tokens::TokenType::Do}); })
    .property("body", [this](NodeInstance& instance){ return parseSingle(); })
    .require([this](NodeInstance& instance){tryconsume({Tokens::TokenType::While}, {"Missing Token", "Expected 'while'"}); return nullptr;})
    .require([this](NodeInstance& instance){tryconsume({Tokens::TokenType::open_paren}, {"Missing Token", "Expected '('"}); return nullptr;})
    .property("expr", [this](NodeInstance& instance){ return parseExpr(new Type{Type::Builtins::BOOLEAN}); })
    .require([this](NodeInstance& instance){tryconsume({Tokens::TokenType::close_paren}, {"Missing Token", "Expected ')'"}); return nullptr;})
    .require([this](NodeInstance& instance){tryconsume({Tokens::TokenType::semicolon}, {"Missing Token", "Expected ';'"}); return nullptr;})
    .registerNode(this->nodes);

    Node::Node(NodeId::for_stmt, [this](){return tryconsume({Tokens::TokenType::For});})
    .require([this](NodeInstance& instance){tryconsume({Tokens::TokenType::open_paren}, {"Missing Token", "Expected '('"}); return nullptr;})
    .property("variable", [this](NodeInstance& instance){ return parseVar(); })
    .require([this](NodeInstance& instance){tryconsume({Tokens::TokenType::semicolon}, {"Missing Token", "Expected ';'"}); return nullptr;})
    .property("expr", [this](NodeInstance& instance){ return parseExpr(new Type{Type::Builtins::BOOLEAN}); })
    .require([this](NodeInstance& instance){tryconsume({Tokens::TokenType::semicolon}, {"Missing Token", "Expected ';'"}); return nullptr;})
    .property("incr", [this](NodeInstance& instance){ return parseSingle(); })
    .require([this](NodeInstance& instance){tryconsume({Tokens::TokenType::close_paren}, {"Missing Token", "Expected ')'"}); return nullptr;}).require([this](NodeInstance& instance){tryconsume({Tokens::TokenType::close_paren}, {"Missing Token", "Expected ')'"}); return nullptr;})
    .property("body", [this](NodeInstance& instance){ return parseSingle(); })
    .registerNode(this->nodes);
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
      tryconsume({Tokens::TokenType::open_angle}, {"Missing Token", "Expected '<'"});
      Type* ret = parseType();
      if (tryconsume({Tokens::TokenType::close_angle}))
        return new Type(Type::Builtins::INTERFACE, mut, string(), nullptr, {}, {}, ret);
      tryconsume({Tokens::TokenType::pipe}, {"Missing Token", "Expected ';'"});
      vector<Type*> params;
      bool found = doUntilFind({Tokens::TokenType::close_angle}, [this, &params](){
        params.push_back(parseType());
      }, {Tokens::TokenType::comma}, {"Missing Token", "Expected ','"});
      return new Type(Type::Builtins::INTERFACE, mut, string(), nullptr, {}, params, ret);
    } else if (peek().type == Tokens::TokenType::identifier) {
      string name = decodeIdentifier().value;
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

  Node::Expression *Parser::parseExpr(Type* requiredType) { //TODO
    return nullptr;
  }

  vector<Tokens::Token> Parser::parseFile(string path, string fieldName) {
    stringstream content;
    ifstream ifile{path};
    if (!ifile.good())
      error({"File Error", "Cannot open file"});
    string buf;
    while (getline(ifile, buf))
      content << buf << "\n";
    ifile.close();
    Tokenizer::Tokenizer tokenizer(content.str());
    vector<Tokens::Token> tokens = tokenizer.tokenize();
    vector<Tokens::Token> publics;
    bool skip = false;
    for (int i = 0; i < tokens.size(); i++) {
      Tokens::Token token = tokens[i];
      if (token.type == Tokens::TokenType::Public) {
        Tokens::Token name = tokens[++i];
        if (name.type != Tokens::TokenType::identifier)
          error({"Internal Error", "Syntax Error in imported file"});

        skip = name.value != fieldName;
        Tokens::Token bracket = tokens[++i];
        if (bracket.type != Tokens::TokenType::public_closure)
          error({"Internal Error", "Syntax Error in imported file"});
        i++;
        while (tokens[i].type != Tokens::TokenType::public_closure) {
          if (!skip) {
            Tokens::Token cur = tokens[i];
            cur.line = getCurrentLine();
            publics.push_back(cur);
          }
          i++;
        }
      }
      i++;
    }

    if (skip)
      error({"Syntax Error", "Imported field does not exist"});
    
    return publics;
  }

  Tokens::Token Parser::getIdentifier() {
    Tokens::Token ident = decodeIdentifier();
    stringstream ss;
    for (string s : this->namespaces)
      ss << s << ":";
    ss << ident.value;
    ident.value = ss.str();
    return ident;
  }

  Tokens::Token Parser::decodeIdentifier() {
    Tokens::Token t = peek();
    Tokens::Token ident = tryconsume({Tokens::TokenType::identifier}, {"Missing Token", "Expected Identifier"});
    stringstream name;
    name << ident.value;
    while (tryconsume({Tokens::TokenType::d_colon})) {
      Tokens::Token id = tryconsume({Tokens::TokenType::identifier}, {"Missing Token", "Expected Identifier"});
      name << ":" << id.value;
    }
    ident.value = name.str();
    return ident;
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

  Variable *Parser::getVar(Node::Variable *var, vector<Node::Variable *> &variables) {
    for (Variable* v : variables) {
      if (v->name == var->name)
        return v;
    }
    error({"Initial Definition Error", "Variable does not exists"});
  }
  int Parser::findOperation(Operation op, vector<Operation> &operations) {
    for (int i = 0; i < operations.size(); i++) {
      if (operations.at(i) == op)
        return i;
    }
    return -1;
  }

  int Parser::findCast(Node::Cast cast, vector<Node::Cast> &casts) {
    for (int i = 0; i < casts.size(); i++) {
      if (casts.at(i) == cast)
        return i;
    }
    return -1;
  }
}
