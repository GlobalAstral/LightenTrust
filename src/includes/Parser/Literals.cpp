#include <Parser/Literals.hpp>
#include "Literals.hpp"

Lits::Literal::Literal(string value) {
  if (value == "true" || value == "false") {
    this->type = Literal::Type::BOOLEAN;
    this->u.b = value == "true";
    return;
  }
  char suffix = value.back();
  char first = value.front();
  if (suffix == first && suffix == '"') {
    this->type = Literal::Type::STRING;
    string substr = value.substr(1, value.size()-1);
    char* str = (char*)malloc(substr.size()*sizeof(char));
    memcpy(str, substr.c_str(), substr.size());
    this->u.s = str;
    return;
  }
  if (suffix == first && suffix == '\'') {
    this->type = Literal::Type::CHAR;
    char c = value.at(1);
    this->u.c = c;
    return;
  }
  bool dotted = StringUtils::isInString('.', value);
  string num = string(value.c_str());
  if (StringUtils::isInString(suffix, Constants::LITERAL_PREFIXES))
    num = value.substr(0, value.size()-1);

  if (dotted) {
    if (suffix == Constants::LITERAL_LONG || suffix == Constants::LITERAL_OCTAL || suffix == Constants::LITERAL_HEX) {
      this->type = Literal::Type::null;
      return;
    }
    if (suffix == Constants::LITERAL_FLOAT) {
      this->type = Literal::Type::FLOAT;
      this->u.f = stof(num, NULL);
      return;
    }
    if (suffix == Constants::LITERAL_BINARY) {
      double d = stod(num, NULL);
      long long IEEE = *((long long*)&d);
      this->type = Literal::Type::LONG;
      this->u.l = IEEE;
      return;
    }
    this->type = Literal::Type::DOUBLE;
    this->u.d = stod(num, NULL);
    return;
  }

  if (suffix == Constants::LITERAL_LONG) {
    this->type = Literal::Type::LONG;
    this->u.l = stoll(num, NULL, 10);
  } else if (suffix == Constants::LITERAL_FLOAT) {
    this->type = Literal::Type::FLOAT;
    this->u.f = stof(num, NULL);
  } else if (suffix == Constants::LITERAL_DOUBLE) {
    this->type = Literal::Type::DOUBLE;
    this->u.d = stod(num, NULL);
  } else if (suffix == Constants::LITERAL_BINARY) {
    this->type = Literal::Type::INT;
    this->u.i = stoi(num, NULL, 2);
  } else if (suffix == Constants::LITERAL_OCTAL) {
    this->type = Literal::Type::INT;
    this->u.i = stoi(num, NULL, 8);
  } else if (suffix == Constants::LITERAL_HEX) {
    this->type = Literal::Type::INT;
    this->u.i = stoi(num, NULL, 16);
  } else {
    this->type = Literal::Type::INT;
    this->u.i = stoi(num, NULL);
  }
}
Lits::Literal::Type Lits::Literal::getType() {
  return this->type;
}

std::ostream &Lits::operator<<(std::ostream &stream, const Literal &t) {
  switch (t.type) {
    case Literal::Type::BOOLEAN :
      stream << "BOOLEAN(" << t.u.b << ")";
      break;
    case Literal::Type::CHAR :
      stream << "CHAR(" << t.u.c << ")";
      break;
    case Literal::Type::DOUBLE :
      stream << "DOUBLE(" << t.u.d << ")";
      break;
    case Literal::Type::FLOAT :
      stream << "FLOAT(" << t.u.f << ")";
      break;
    case Literal::Type::INT :
      stream << "INT(" << t.u.i << ")";
      break;
    case Literal::Type::LONG :
      stream << "LONG(" << t.u.l << ")";
      break;
    case Literal::Type::STRING :
      stream << "STRING(" << t.u.s << ")";
      break;
    case Literal::Type::null :
      stream << "NULL";
      break;
    default:
      stream << "DEFAULT (HOW?)";
  }
  return stream;
}
