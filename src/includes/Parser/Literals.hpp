#pragma once

#include <iostream>
#include <Utils/StringUtils.hpp>
#include <Utils/Constants.hpp>
namespace Lits {  
  
  using namespace std;
  class Literal {
    public:
      enum class Type {
        INT, LONG, FLOAT, DOUBLE, CHAR, BOOLEAN, STRING, null,
      };
      Literal(string value);
    private:
      union {
        int i;
        long long l;
        float f;
        double d;
        unsigned char c;
        bool b;
        char* s;
      } u;
      Type type;
  };
}  

