#include <Generator/Generator.hpp>
#include "Generator.hpp"

namespace Generator {
  void Generator::print(std::ostream &stream) {
    stream << output.str();
  }

  std::string Generator::getOutput() {
    output.str("");
    output << sec_bss.str() << "\n\n" << sec_data.str() << "\n\n" << labels.str() << "\n\n" << sec_text.str();
    return output.str();
  }

  Long Generator::getSizeof(Type *t) {
    if (t == nullptr || t->type == Type::Builtins::VOID)
      error({"SizeError", "Cannot get size of incomplete type"});

    switch (t->type) {
      case Type::Builtins::ALIAS :
        return getSizeof((*declaredTypes)[t->identifier]);
      case Type::Builtins::BOOLEAN :
      case Type::Builtins::CHAR :
      case Type::Builtins::BYTE :
        return 1;
      case Type::Builtins::FLOAT :
      case Type::Builtins::INT :
      case Type::Builtins::UINT :
        return 4;
      case Type::Builtins::DOUBLE :
      case Type::Builtins::LONG :
      case Type::Builtins::ULONG :
      case Type::Builtins::INTERFACE :
      case Type::Builtins::POINTER :
      case Type::Builtins::STRING : 
        return 8;
      case Type::Builtins::STRUCT : {
        int acc = 0;
        for (Variable* field : t->fields)
          acc += getSizeof(field->t);
        return acc;
      }
      case Type::Builtins::UNION : {
        int max = -1;
        for (Variable* field : t->fields) {
          int acc = getSizeof(field->t);
          if (acc > max)
            max = acc;
        }
        return max;
      }
      default:
        error({"SizeError", "Cannot get size of non-existent type"});
    }
  }
}
