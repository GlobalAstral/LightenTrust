#include <Parser/Node.hpp>
#include "Node.hpp"

using std::any, std::string;
using namespace Node;

Node::Property::Property(string name, function<any(NodeInstance &)> criteria) {
  this->name = name;
  this->criteria = criteria;
};

Node::Property::Property(function<any(NodeInstance &)> criteria) {
  this->name = string();
  this->criteria = criteria;
}

any Node::Property::get(){
  if (!validValue)
    Errors::error("Illegal State", "Cannot get value before it is valid");
  return value;
}

string Node::Property::getName() {
  return this->name;
}

bool Node::Property::isValid() {
  return false;
}

void Node::Property::invoke(NodeInstance &instance) {
  if (validValue)
    Errors::error("Illegal State", "Cannot invoke criteria with valid value");
  this->value = criteria(instance);
  validValue = true;
}

void Node::NodeInstance::print(std::ostream &stream) {
  stream << this->id << " ";
  onPrint(stream);
}

Node::Node::Node() { }

Node::Node::Node(NodeId name, function<bool()> criteria) {
  this->id = name;
  this->criteria = criteria;
}

bool Node::Node::check() { return criteria(); }

bool Node::Node::doAdd() {return !doNotAdd;}

Node::Node &Node::Node::property(string name, function<any(NodeInstance &instance)> f) {
  Property* prop = new Property(name, f);
  requirements.push_back(prop);
  return *this;
}

Node::Node &Node::Node::require(function<void *(NodeInstance &instance)> f) {
  Property* prop = new Property(f);
  requirements.push_back(prop);
  return *this;
}

Node::Node &Node::Node::finally(function<void(NodeInstance &)> fnal) {
  this->fnal = fnal;
  return *this;
}

Node::Node &Node::Node::onPrint(function<void(NodeInstance &, std::ostream &)> onprint) {
  this->on_print = onprint;
  return *this;
}

Node::Node &Node::Node::notAdd() {
  this->doNotAdd = true;
  return *this;
}

void Node::Node::registerNode(vector<Node> &nodes) {
  nodes.push_back(*this);
}

NodeInstance *Node::Node::build() {
  NodeInstance* ret = new NodeInstance();
  ret->id = id;
  ret->add = this->doAdd();
  auto on_print_copy = on_print;
  ret->onPrint = [on_print_copy, ret](std::ostream& stream) -> void {
    on_print_copy(*ret, stream);
  };
  for (Property* prop : requirements) {
    ret->requirements.push_back(new Property(*prop));
  }

  for (Property* prop : ret->requirements) {
    prop->invoke(*ret);
  }
  this->fnal(*ret);
  return ret;
}

Node::Type::Type(Builtins type, bool mut, string identifier, Type *pointsTo, vector<Variable *> fields, vector<Type *> params, Type *retType, Expression *expr) {
  this->type = type;
  this->mut = mut;
  this->identifier = identifier;
  this->pointsTo = pointsTo;
  this->fields = fields;
  this->params = params;
  this->returnType = retType;
  this->initSize = expr;
}

bool Node::Type::operator==(const Type &a) const {
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

bool Node::Type::operator!=(const Type &a) const {
  return !(*this == a);
}

std::ostream &Node::operator<<(std::ostream &out, NodeId id) {
  switch (id) {
    case NodeId::scope:
      out << "SCOPE";
      break;
    case NodeId::func_decl:
      out << "FUNC DECL";
      break;
    case NodeId::var_decl:
      out << "VAR DECL";
      break;
    case NodeId::type_decl:
      out << "TYPE DECL";
      break;
    case NodeId::public_field:
      out << "PUBLIC FIELD";
      break;
    case NodeId::import:
      out << "IMPORT";
      break;
    case NodeId::namesp:
      out << "NAMESPACE";
      break;
    case NodeId::defer:
      out << "DEFER";
      break;
    case NodeId::var_set:
      out << "VAR SET";
      break;
    case NodeId::return_stmt:
      out << "RETURN";
      break;
    case NodeId::asm_code:
      out << "ASM";
      break;
    case NodeId::operation_decl:
      out << "OPERATION DECL";
      break;
    case NodeId::cast_decl:
      out << "CAST DECL";
      break;
    case NodeId::if_stmt:
      out << "IF";
      break;
    case NodeId::while_stmt:
      out << "WHILE";
      break;
    case NodeId::do_while_stmt:
      out << "DO WHILE";
      break;
    case NodeId::for_stmt:
      out << "FOR";
      break;
    case NodeId::alias_decl:
      out << "ALIAS DECL";
      break;
    case NodeId::alias_use:
      out << "ALIAS";
      break;
    default:
      out << "NULL";
  }
  return out;
}

std::ostream &Node::operator<<(std::ostream &stream, const Type &t) {
  stream << (t.mut ? "MUTABLE" : "") << " ";
  switch (t.type) {
    break;
    case Type::Builtins::ALIAS :
      stream << "ALIAS";
      break;
    case Type::Builtins::BOOLEAN :
      stream << "BOOLEAN";
      break;
    case Type::Builtins::BYTE :
      stream << "BYTE";
      break;
    case Type::Builtins::CHAR :
      stream << "CHAR";
      break;
    case Type::Builtins::DOUBLE :
      stream << "DOUBLE";
      break;
    case Type::Builtins::FLOAT :
      stream << "FLOAT";
      break;
    case Type::Builtins::INT :
      stream << "INT";
      break;
    case Type::Builtins::INTERFACE :
      stream << "INTERFACE<(";
      for (Type* pType : t.params)
        stream << *pType << ",";
      stream << ") -> " << *(t.returnType);
      stream << ">";
      break;
    case Type::Builtins::LONG :
      stream << "LONG";
      break;
    case Type::Builtins::POINTER :
      stream << "*" << *t.pointsTo;
      break;
    case Type::Builtins::STRING :
      stream << "STRING";
      break;
    case Type::Builtins::STRUCT :
      stream << "STRUCT{";
      for (Variable* var : t.fields)
        stream << *(var->t) << " : " << var->name << ";\n";
      stream << "}";
      break;
    case Type::Builtins::UNION :
      stream << "UNION{";
      for (Variable* var : t.fields)
        stream << *(var->t) << " : " << var->name << ";\n";
      stream << "}";
      break;
    case Type::Builtins::UINT :
      stream << "UINT";
      break;
    case Type::Builtins::ULONG :
      stream << "ULONG";
      break;
    case Type::Builtins::VOID :
      stream << "VOID";
      break;
    default:
      stream << "NULL";
  }

  if (!t.identifier.empty())
    stream << "(" << t.identifier << ")";

  return stream;
}

std::ostream &Node::operator<<(std::ostream &stream, const ExprType etype) {
  switch (etype) {
    case ExprType::cast :
      stream << "CAST";
      break;
    case ExprType::custom :
      stream << "CUSTOM";
      break;
    case ExprType::dereference :
      stream << "DEREF";
      break;
    case ExprType::dot_notation :
      stream << "DOT NOTATION";
      break;
    case ExprType::func_call :
      stream << "FUNC CALL";
      break;
    case ExprType::interface_ref :
      stream << "INTERFACE REF";
      break;
    case ExprType::literal :
      stream << "LITERAL";
      break;
    case ExprType::reference :
      stream << "REF";
      break;
    case ExprType::subscript :
      stream << "SUBSCRIPT";
      break;
    case ExprType::variable :
      stream << "VAR";
      break;
    case ExprType::interface_call :
      stream << "INTERFACE CALL";
      break;
    default:
      stream << "NULL";
  }
  return stream;
}

std::ostream &Node::operator<<(std::ostream &stream, const Expression &e) {
  struct Visitor {
    std::ostream& os;
    void operator()(Lits::Literal* literal) const {
      os << *literal;
    }
    void operator()(Variable* var) const {
      os << var->name << " : " << *(var->t);
    }
    void operator()(NodeInstance* instance) const {
      instance->print(os);
    }
    void operator()(Expression* expr) const {
      os << *expr;
    }
    void operator()(SubscriptExpr* expr) const {
      os << *(expr->base) << "[" << *(expr->index) << "]";
    }
    void operator()(vector<Variable*> variables) const {
      for (int i = 0; i < variables.size(); i++) {
        if (i > 0)
          os << '.';
        Variable* var = variables[i];
        os << "(" << var->name << " : " << *(var->t) << ")";
      }
    }
    void operator()(FuncCall* func_call) const {
      func_call->func->print(os);
      os << " WITH PARAMS (";
      for (Expression* e : func_call->params) {
        os << *e << ", ";
      }
      os << ")";
    }
    void operator()(InterfaceCall* interface_call) const {
      os << *(interface_call->interface) << " WITH PARAMS (";
      for (Expression* e : interface_call->params) {
        os << *e << ", ";
      }
      os << ")";
    }
    void operator()(DotNotation* exprs) const {
      os << "(" <<  *(exprs->base) << ")" << ".(" << *(exprs->after) << ")";
    }
    void operator()(CastExpr* cast) const {
      os << *(cast->expr) << "[" << *(cast->cast->a) << "]" << " as " << *(cast->cast->b) << " : ";
      cast->cast->body->print(os);
    }
    void operator()(CustomExpr* custom) const {
      os << (custom->op->unary ? "UNARY " : "BINARY ");
      os << *(custom->a) << "[" << *(custom->op->a) << "] " << custom->op->symbols << " " << *(custom->b) << "[" << *(custom->op->b) << "]" << "<" << custom->op->precedence << "> : ";
      custom->op->body->print(os);
    }
  };

  stream << e.type << " -> " << *(e.returnType) << " : ";
  std::visit(Visitor{stream}, e.variant);

  return stream;
}

bool Node::Operation::operator==(const Operation &a) const {
  if (unary != a.unary || symbols != a.symbols || (*(this->a) != *(a.a)) || (*(this->b) != *(a.b)) || (*(this->r) != *(a.r))) 
    return false;
  return true;
}

bool Node::Operation::operator!=(const Operation &a) const {
  return !(*this == a);
}

bool Node::Cast::operator==(const Cast &a) const {
  return (*(this->a) == *(a.a) && *(this->b) == *(a.b));
}
