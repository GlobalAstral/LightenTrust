#include <Preprocessor/Preprocessor.hpp>
#include "Preprocessor.hpp"

Preprocessor::Preprocessor::Preprocessor(std::vector<Tokens::Token> tokens) {
  this->content = tokens;
}

void Preprocessor::Preprocessor::print(std::ostream &stream) {
  for (Tokens::Token t : this->output) {
    t.print(stream);
    stream << "\n";
  }
}

std::vector<Tokens::Token> Preprocessor::Preprocessor::preprocess() {
  while (hasPeek()) {
    preprocessSingle(output);
  }
  return output;
}

void Preprocessor::Preprocessor::preprocessSingle(std::vector<Tokens::Token>& out) {
  if (tryconsume({Tokens::TokenType::preprocessor})) {
    if (tryconsume({Tokens::TokenType::define})) {
      std::string name = getIdentifier();
      mustBeUnique(name);
      std::vector<Tokens::Token> body;
      doUntilFind({Tokens::TokenType::preprocessor}, [this, &body](){
        body.push_back(consume());
      }, {"Missing Token", "Expected '#'"});
      definitions[name] = body;
    } else if (tryconsume({Tokens::TokenType::macro})) {
      std::string name = getIdentifier();
      mustBeUnique(name);
      tryconsume({Tokens::TokenType::open_paren}, {"Missing Token", "Expected '('"});
      std::vector<std::string> params;
      doUntilFind({Tokens::TokenType::close_paren}, [this, &params](){
        params.push_back(getIdentifier());
      }, {Tokens::TokenType::comma}, {"Missing Token", "Expected ','"}, {"Missing Token", "Expected ')'"});
      std::vector<Tokens::Token> body;
      doUntilFind({Tokens::TokenType::preprocessor}, [this, &body](){
        body.push_back(consume());
      }, {"Missing Token", "Expected '#'"});
      macros[name] = std::pair<std::vector<std::string>, std::vector<Tokens::Token>>(params, body);
    } else if (tryconsume({Tokens::TokenType::keyword})) {
      std::string name = getIdentifier();
      mustBeUnique(name);
      tryconsume({Tokens::TokenType::open_angle}, {"Missing Token", "Expected '<'"});
      Tokens::Token word = consume();
      tryconsume({Tokens::TokenType::close_angle}, {"Missing Token", "Expected '>'"});
      std::vector<Tokens::Token> body;
      doUntilFind({Tokens::TokenType::preprocessor}, [this, &body](){
        body.push_back(consume());
      }, {"Missing Token", "Expected '#'"});
      keywords[name] = std::pair<Tokens::Token, std::vector<Tokens::Token>>(word, body);
    } else if (tryconsume({Tokens::TokenType::_template})) {
      std::string name = getIdentifier();
      Template templ;
      mustBeUnique(name);
      tryconsume({Tokens::TokenType::open_angle}, {"Missing Token", "Expected '<'"});
      doUntilFind({Tokens::TokenType::close_angle}, [this, &templ](){
        templ.generics.push_back(getIdentifier());
      }, {Tokens::TokenType::comma}, {"Missing Token", "Expected ','"}, {"Missing Token", "Expected '>'"});

      tryconsume({Tokens::TokenType::open_paren}, {"Missing Token", "Expected '('"});
      doUntilFind({Tokens::TokenType::close_paren}, [this, &templ](){
        templ.params.push_back(getIdentifier());
      }, {Tokens::TokenType::comma}, {"Missing Token", "Expected ','"}, {"Missing Token", "Expected ')'"});
      
      tryconsume({Tokens::TokenType::open_square}, {"Missing Token", "Expected '['"});
      templ.body = getIdentifier();
      tryconsume({Tokens::TokenType::close_square}, {"Missing Token", "Expected ']'"});

      doUntilFind({Tokens::TokenType::preprocessor}, [this, &templ](){
        templ.content.push_back(consume());
      }, {"Missing Token", "Expected '#'"});
      templates[name] = templ;
    } else if (tryconsume({Tokens::TokenType::undef})) {
      std::string name = getIdentifier();
      if (definitions.contains(name)) {
        definitions.remove(name);
      } else if (macros.contains(name)) {
        macros.remove(name);
      } else if (keywords.contains(name)) {
        keywords.remove(name);
      } else {
        error({"Syntax Error", "Definition does not exist"});
      }
    } else if (peek().type == Tokens::TokenType::ifdef || peek().type == Tokens::TokenType::ifndef) {
      bool negative = consume().type == Tokens::TokenType::ifndef;
      std::string name = getIdentifier();
      bool unique = isUnique(name);
      bool ignore = unique && !negative || !unique && negative;
      doUntilFind({Tokens::TokenType::endif}, [this, ignore, &out](){
        if (ignore) {
          consume();
          return;
        }
        preprocessSingle(out);
      }, {"Missing Token", "Expected '#'"});
    } else if (peek().type == Tokens::TokenType::logi || peek().type == Tokens::TokenType::logw || peek().type == Tokens::TokenType::loge) {
      enum class error_type {
        info, warning, error
      };
      error_type t = tryconsume({Tokens::TokenType::logi}) ? error_type::info : tryconsume({Tokens::TokenType::logw}) ? error_type::warning : error_type::error;
      if (t == error_type::error) consume();
      std::stringstream ss;
      doUntilFind({Tokens::TokenType::preprocessor}, [this, &ss](){
        ss << consume().value << ' ';
      }, {"Missing Token", "Expected '#'"});
      switch (t) {
        case error_type::info :
          Errors::info(ss.str());  
          break;
        case error_type::warning :
          Errors::warn(ss.str());  
          break;
        case error_type::error :
          error({"Directive Error", ss.str()});
      };
    } else {
      error({"Syntax Error", "Unknown preprocessor directive"});
    }
  } else if (peek().type == Tokens::TokenType::identifier) {
    Tokens::Token ident = consume();
    while (tryconsume({Tokens::TokenType::at})) {
      std::string s = getIdentifier();
      if (definitions.contains(s)) {
        for (Tokens::Token t : definitions[s])
          ident.value += std::string(t.value);
      } else if (internal.contains(s)) {
        for (Tokens::Token t : internal[s])
          ident.value += std::string(t.value);
      } else {
        ident.value += std::string(s);
      }
    }
    std::string name = ident.value;
    if (definitions.contains(name)) {
      withTokens(definitions[name], [this, &out](std::vector<Tokens::Token>& oldTokens, int& old_peek){
        preprocess(out);
      });
    } else if (internal.contains(name)) {
      withTokens(internal[name], [this, &out](std::vector<Tokens::Token>& oldTokens, int& old_peek){
        preprocess(out);
      });
    } else if (keywords.contains(name)) {
      Tokens::Token param = consume();
      std::pair<Tokens::Token, std::vector<Tokens::Token>> keyword = keywords[name];
      internal[keyword.first.value] = {param};
      withTokens(keyword.second, [this, &out](std::vector<Tokens::Token>& oldTokens, int& old_peek){
        preprocess(out);
      });
      internal.remove(keyword.first.value);
    } else if (macros.contains(name)) {
      std::pair<std::vector<std::string>, std::vector<Tokens::Token>> macro = macros[name];
      std::vector<std::string> params = macro.first;
      std::vector<Tokens::Token> body = macro.second;
      std::vector<std::vector<Tokens::Token>> params_expr;
      tryconsume({Tokens::TokenType::open_paren}, {"Missing Token", "Expected '('"});
      if (params.empty()) 
        tryconsume({Tokens::TokenType::close_paren}, {"Missing Token", "Expected ')'"});
      else {
        std::vector<Tokens::Token> buffer;
        doUntilFind({Tokens::TokenType::close_paren}, [this, &params_expr, &buffer](){
          if (tryconsume({Tokens::TokenType::comma})) {
            params_expr.push_back(std::vector<Tokens::Token>(buffer));
            buffer.clear();
          } else {
            buffer.push_back(consume());
          }
        }, {"Missing Token", "Expected ')'"});

        params_expr.push_back(buffer);
      }

      if (params_expr.size() != params.size())
        error({"Syntax Error", "Macro parameters mismatch"});
        
      for (int i = 0; i < params.size(); i++)
        internal[params[i]] = params_expr[i];
      
      withTokens(body, [this, &out](std::vector<Tokens::Token>& oldTokens, int& old_peek){
        preprocess(out);
      });
      
      for (std::string param : params)
        internal.remove(param);
    } else if (templates.contains(name)) {
      Template templ = templates[name];
      std::vector<std::vector<Tokens::Token>> generics_value;
      std::vector<std::vector<Tokens::Token>> params_value;
      std::vector<Tokens::Token> body_value;
      if (!templ.generics.empty()) {
        tryconsume({Tokens::TokenType::open_angle}, {"Missing Token", "Expected '<'"});
        std::vector<Tokens::Token> buffer;
        doUntilFind({Tokens::TokenType::close_angle}, [this, &generics_value, &buffer](){
          if (tryconsume({Tokens::TokenType::comma})) {
            generics_value.push_back(std::vector<Tokens::Token>(buffer));
            buffer.clear();
          } else {
            buffer.push_back(consume());
          }
        }, {"Missing Token", "Expected '>'"});

        generics_value.push_back(buffer);
      }
      if (!templ.params.empty()) {
        tryconsume({Tokens::TokenType::open_paren}, {"Missing Token", "Expected '('"});
        std::vector<Tokens::Token> buffer;
        doUntilFind({Tokens::TokenType::close_paren}, [this, &params_value, &buffer](){
          if (tryconsume({Tokens::TokenType::comma})) {
            params_value.push_back(std::vector<Tokens::Token>(buffer));
            buffer.clear();
          } else {
            buffer.push_back(consume());
          }
        }, {"Missing Token", "Expected ')'"});

        params_value.push_back(buffer);
      }
      tryconsume({Tokens::TokenType::open_curly}, {"Missing Token", "Expected '{'"});
      doUntilFind({Tokens::TokenType::close_curly}, [this, &body_value](){
        body_value.push_back(consume());
      }, {"Missing Token", "Expected '}'"});

      if (generics_value.size() != templ.generics.size())
        error({"Syntax Error", "Macro generics mismatch"});
      if (params_value.size() != templ.params.size())
        error({"Syntax Error", "Macro parameters mismatch"});
      
      for (int i = 0; i < generics_value.size(); i++)
        internal[templ.generics[i]] = generics_value[i];
      for (int i = 0; i < params_value.size(); i++)
        internal[templ.params[i]] = params_value[i];
      internal[templ.body] = body_value;

      withTokens(templ.content, [this, &out](std::vector<Tokens::Token>&, int&) {
        preprocess(out); 
      });

      for (std::string generic : templ.generics)
        internal.remove(generic);
      for (std::string param : templ.params)
        internal.remove(param);
      internal.remove(templ.body);
    } else {
      out.push_back(ident);
    }
  } else {
    out.push_back(consume());
  }
}

bool Preprocessor::Preprocessor::isUnique(std::string name) {
  return !definitions.contains(name) && !macros.contains(name) && !keywords.contains(name) && !templates.contains(name);
}

void Preprocessor::Preprocessor::mustBeUnique(std::string name) {
  if (!isUnique(name))
    error({"Syntax Error", "Definition already exists"});
}

void Preprocessor::Preprocessor::mustExist(std::string name) {
  if (isUnique(name))
    error({"Syntax Error", "Definition does not exist"});
}

std::string Preprocessor::Preprocessor::getIdentifier() {
  return tryconsume({Tokens::TokenType::identifier}, {"Missing Token", "Expected Identifier"}).value;
}

void Preprocessor::Preprocessor::withTokens(std::vector<Tokens::Token>& newTokens, int newPeek, std::function<void(std::vector<Tokens::Token>&, int&)> lambda) {
  int old_peek = this->_peek;
  auto old_tokens = this->content;

  this->content = newTokens;
  this->_peek = newPeek;

  lambda(old_tokens, old_peek);

  this->content = std::move(old_tokens);
  this->_peek = old_peek;
}

void Preprocessor::Preprocessor::withTokens(std::vector<Tokens::Token>& newTokens, std::function<void(std::vector<Tokens::Token>&, int&)> lambda) {
  this->withTokens(newTokens, 0, lambda);
}

void Preprocessor::Preprocessor::preprocess(std::vector<Tokens::Token> &out) {
  while (hasPeek()) {
    preprocessSingle(out);
  }
}

Tokens::Token Preprocessor::Preprocessor::null() {
  return Tokens::nullToken();
}
int Preprocessor::Preprocessor::getCurrentLine() {
  return peek(-1).line;
}

std::string Preprocessor::Preprocessor::getCurrentColumn() { 
  std::stringstream ss;
  peek(-1).print(ss);
  return ss.str(); 
}
bool Preprocessor::Preprocessor::equalCriteria(Tokens::Token a, Tokens::Token b) {
  if (a.type != b.type || (!a.value.empty() && !b.value.empty() && a.value != b.value))
    return false;
  return true;
};
