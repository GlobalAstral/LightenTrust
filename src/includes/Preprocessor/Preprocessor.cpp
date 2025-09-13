#include <Preprocessor/Preprocessor.hpp>

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
    } else if (tryconsume({Tokens::TokenType::undef})) {
      //TODO INFO WARNING ERROR (MAYBE PRAGMA) (MAYBE DIRECTIVE WITH PLACEHOLDERS)
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
      withTokens(definitions[name], [this, &out](){
        while (hasPeek())
          preprocessSingle(out);
      });
    } else if (internal.contains(name)) {
      withTokens(internal[name], [this, &out](){
        while (hasPeek())
          preprocessSingle(out);
      });
    } else if (keywords.contains(name)) {
      Tokens::Token param = consume();
      std::pair<Tokens::Token, std::vector<Tokens::Token>> keyword = keywords[name];
      internal[keyword.first.value] = {param};
      withTokens(keyword.second, [this, &out](){
        while (hasPeek())
          preprocessSingle(out);
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
      
      withTokens(body, [this, &out](){
        while (hasPeek())
          preprocessSingle(out);
      });
      
      for (std::string param : params)
        internal.remove(param);
    } else {
      out.push_back(ident);
    }
  } else {
    out.push_back(consume());
  }
}

bool Preprocessor::Preprocessor::isUnique(std::string name) {
  return !definitions.contains(name) && !macros.contains(name) && !keywords.contains(name);
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

void Preprocessor::Preprocessor::withTokens(std::vector<Tokens::Token>& newTokens, int newPeek, std::function<void()> lambda) {
  int old_peek = this->_peek;
  auto old_tokens = std::move(this->content);

  this->content = newTokens;
  this->_peek = newPeek;

  lambda();

  this->content = std::move(old_tokens);
  this->_peek = old_peek;
}

void Preprocessor::Preprocessor::withTokens(std::vector<Tokens::Token>& newTokens, std::function<void()> lambda) {
  this->withTokens(newTokens, 0, lambda);
}

Tokens::Token Preprocessor::Preprocessor::null()
{
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
