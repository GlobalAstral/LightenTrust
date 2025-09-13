#pragma once

#include <iostream>
#include <vector>
#include <functional>

#include <Utils/Errors.hpp>
#include <sstream>

namespace Processor {
  template <typename T>
  class Processor {
    protected:
      int _peek = 0;
      std::vector<T> content{};
      
      virtual T null() { error({"Internal Error", "Cannot call 'getCurrentLine' from Processor"});  };
      virtual int getCurrentLine() { error({"Internal Error", "Cannot call 'getCurrentLine' from Processor"}); return -1;}; //? dummy value, to satisfy compiler
      virtual bool equalCriteria(T a, T b) = 0;
      virtual std::string getCurrentColumn() { error({"Internal Error", "Cannot call 'getCurrentColumn' from Processor"}); return "";}; //? dummy value, to satisfy compiler

      bool hasPeek(int offset = 0) { return _peek+offset >= 0 && _peek+offset < content.size(); }
      T peek(int offset = 0) { return (hasPeek(offset) ? this->content[_peek+offset] : null()); }
      T consume() { return (hasPeek() ? content[_peek++] : null()); }
      void consume(int amount) { for (int i = 0; i < amount; i++) {consume();} }
      bool tryconsume(T c) { if (equalCriteria(peek(), c)) {consume();return true;} return false; }
      T tryconsume(T c, Errors::CompactError error) { if (equalCriteria(peek(), c)) {return consume();} this->error(error); }
      [[noreturn]] void error(Errors::CompactError error) {
        std::stringstream ss;
        ss << error.msg; 
        ss << " [";
        ss << getCurrentColumn(); 
        ss << "]";

        Errors::error({error.error, ss.str()}, getCurrentLine());
      }
      bool doUntilFind(T toFind, std::function<void()> func) {
        bool found = false;
        while (hasPeek()) {
          if (tryconsume(toFind)) {
            found = true; 
            break;
          }
          func();
        } 
        return found; 
      };

      void doUntilFind(T toFind, std::function<void()> func, Errors::CompactError notfounderr) {
        bool found = false;
        while (hasPeek()) {
          if (tryconsume(toFind)) {
            found = true; 
            break;
          }
          func();
        } 
        if (!found)
          error(notfounderr);
      };

      bool doUntilFind(T toFind, std::function<void()> func, T sep, Errors::CompactError error) {
        bool found = false;
        while (hasPeek()) {
          if (tryconsume(toFind)) {
            found = true; 
            break;
          }
          func();
          if (tryconsume(toFind)) {
            found = true; 
            break;
          }
          tryconsume(sep, error);
        } 
        return found; 
      };

      void doUntilFind(T toFind, std::function<void()> func, T sep, Errors::CompactError sepnotfound, Errors::CompactError notfounderr) {
        bool found = false;
        while (hasPeek()) {
          if (tryconsume(toFind)) {
            found = true; 
            break;
          }
          func();
          if (tryconsume(toFind)) {
            found = true; 
            break;
          }
          tryconsume(sep, sepnotfound);
        } 
        if (!found)
          error(notfounderr);
      };
  };
}
