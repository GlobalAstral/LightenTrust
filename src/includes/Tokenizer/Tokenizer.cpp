#include <Tokenizer/Tokenizer.hpp>
#include "Tokenizer.hpp"

Tokenizer::Tokenizer::Tokenizer(std::string s) {
  for (char c : s)
    content.push_back(c);
}

std::vector<Tokens::Token> Tokenizer::Tokenizer::tokenize() {
  using std::vector, std::string, std::stringstream;

  vector<Tokens::Token> tokens{};
  bool comment = false;
  bool multi_comment = false;
  
  while (hasPeek()) {
    if (peek() == ' ' || peek() == '\r') {
      consume();
    } else if (peek() == '\n') {
      consume();
      comment = false;
      line++;
    } else if (comment || multi_comment) {
      consume();
    } else if (tryconsume('/')) {
      if (tryconsume('/')) {
        comment = true;
      } else if (tryconsume('*')) {
        multi_comment = true;
      } else {
        tokens.push_back({Tokens::TokenType::symbols, line, "/"});
      }
    } else if (tryconsume('*')) {
      if (tryconsume('/')) {
        multi_comment = false;
      } else {
        tokens.push_back({Tokens::TokenType::symbols, line, "*"});
      }

    } else if (tryconsume('(')) {
      tokens.push_back({Tokens::TokenType::open_paren, line});
    } else if (tryconsume(')')) {
      tokens.push_back({Tokens::TokenType::close_paren, line});
    } else if (tryconsume('{')) {
      tokens.push_back({Tokens::TokenType::open_curly, line});
    } else if (tryconsume('}')) {
      tokens.push_back({Tokens::TokenType::close_curly, line});
    } else if (tryconsume('[')) {
      tokens.push_back({Tokens::TokenType::open_square, line});
    } else if (tryconsume(']')) {
      tokens.push_back({Tokens::TokenType::close_square, line});
    } else if (tryconsume('<')) {
      tokens.push_back({Tokens::TokenType::open_angle, line});
    } else if (tryconsume('>')) {
      tokens.push_back({Tokens::TokenType::close_angle, line});
    } else if (tryconsume(';')) {
      tokens.push_back({Tokens::TokenType::semicolon, line});
    } else if (tryconsume('@')) {
      tokens.push_back({Tokens::TokenType::at, line});
    } else if (tryconsume(':')) {
      if (tryconsume(':'))
        tokens.push_back({Tokens::TokenType::d_colon, line});
      else
        tokens.push_back({Tokens::TokenType::colon, line});
    } else if (tryconsume('.')) {
      tokens.push_back({Tokens::TokenType::dot, line});
    } else if (tryconsume(',')) {
      tokens.push_back({Tokens::TokenType::comma, line});
    } else if (tryconsume('$')) {
      tokens.push_back({Tokens::TokenType::public_closure, line});
    } else if (tryconsume('|')) {
      tokens.push_back({Tokens::TokenType::pipe, line});
    } else if (tryconsume('#')) {
      tokens.push_back({Tokens::TokenType::preprocessor, line});
    } else if (peek() == '-' && peek(1) == '>') {
      tokens.push_back({Tokens::TokenType::arrow, line});
      consume(2);
    } else if (tryconsume('\'')) {
      char c = consume();
      if (!tryconsume('\''))
        Errors::error("Missing token", "Closing single quote expected", line);
      tokens.push_back({Tokens::TokenType::literal, line, StringUtils::parseEscapes("'" + string(1, c) + "'")});
    } else if (tryconsume('"')) {
      stringstream ss;
      ss << '"';
      while (hasPeek() && !tryconsume('"')) {
        if (peek() == '\\') {
          ss << consume();
        }
        ss << consume();
      }
      if (!hasPeek())
        Errors::error("Missing token", "Closing double quote expected", line);
      ss << '"';
      string s = StringUtils::parseEscapes(ss.str());
      tokens.push_back({Tokens::TokenType::literal, line, s});
    } else {
      stringstream buf;
      if (isalpha(peek()) || peek() == '_') {
        while (isalnum(peek()) || peek() == '_')
          buf << consume();
        string buffer = string(buf.str());
        if (buffer == "int") {
          tokens.push_back({Tokens::TokenType::Int, line, buffer});
        } else if (buffer == "uint") {
          tokens.push_back({Tokens::TokenType::Uint, line, buffer});
        } else if (buffer == "float") {
          tokens.push_back({Tokens::TokenType::Float, line, buffer});
        } else if (buffer == "long") {
          tokens.push_back({Tokens::TokenType::Long, line, buffer});
        } else if (buffer == "ulong") {
          tokens.push_back({Tokens::TokenType::Ulong, line, buffer});
        } else if (buffer == "double") {
          tokens.push_back({Tokens::TokenType::Double, line, buffer});
        } else if (buffer == "char") {
          tokens.push_back({Tokens::TokenType::Char, line, buffer});
        } else if (buffer == "byte") {
          tokens.push_back({Tokens::TokenType::Byte, line, buffer});
        } else if (buffer == "string") {
          tokens.push_back({Tokens::TokenType::String, line, buffer});
        } else if (buffer == "void") {
          tokens.push_back({Tokens::TokenType::Void, line, buffer});
        } else if (buffer == "struct") {
          tokens.push_back({Tokens::TokenType::Struct, line, buffer});
        } else if (buffer == "union") {
          tokens.push_back({Tokens::TokenType::Union, line, buffer});
        } else if (buffer == "interface") {
          tokens.push_back({Tokens::TokenType::Interface, line, buffer});
        } else if (buffer == "return") {
          tokens.push_back({Tokens::TokenType::Return, line, buffer});
        } else if (buffer == "mutable") {
          tokens.push_back({Tokens::TokenType::Mutable, line, buffer});
        } else if (buffer == "inline") {
          tokens.push_back({Tokens::TokenType::Inline, line, buffer});
        } else if (buffer == "type") {
          tokens.push_back({Tokens::TokenType::Type, line, buffer});
        } else if (buffer == "if") {
          tokens.push_back({Tokens::TokenType::If, line, buffer});
        } else if (buffer == "else") {
          tokens.push_back({Tokens::TokenType::Else, line, buffer});
        } else if (buffer == "while") {
          tokens.push_back({Tokens::TokenType::While, line, buffer});
        } else if (buffer == "do") {
          tokens.push_back({Tokens::TokenType::Do, line, buffer});
        } else if (buffer == "for") {
          tokens.push_back({Tokens::TokenType::For, line, buffer});
        } else if (buffer == "namespace") {
          tokens.push_back({Tokens::TokenType::Namespace, line, buffer});
        } else if (buffer == "defer") {
          tokens.push_back({Tokens::TokenType::Defer, line, buffer});
        } else if (buffer == "as") {
          tokens.push_back({Tokens::TokenType::As, line, buffer});
        } else if (buffer == "boolean") {
          tokens.push_back({Tokens::TokenType::Boolean, line, buffer});
        } else if (buffer == "func") {
          tokens.push_back({Tokens::TokenType::Func, line, buffer});
        } else if (buffer == "var") {
          tokens.push_back({Tokens::TokenType::Var, line, buffer});
        } else if (buffer == "public") {
          tokens.push_back({Tokens::TokenType::Public, line, buffer});
        } else if (buffer == "import") {
          tokens.push_back({Tokens::TokenType::import, line, buffer});
        } else if (buffer == "true") {
          tokens.push_back({Tokens::TokenType::literal, line, "true"});
        } else if (buffer == "false") {
          tokens.push_back({Tokens::TokenType::literal, line, "false"});
        } else if (buffer == "below") {
          tokens.push_back({Tokens::TokenType::below, line, buffer});
        } else if (buffer == "above") {
          tokens.push_back({Tokens::TokenType::above, line, buffer});
        } else if (buffer == "all") {
          tokens.push_back({Tokens::TokenType::all, line, buffer});
        } else if (buffer == "none") {
          tokens.push_back({Tokens::TokenType::none, line, buffer});
        } else if (buffer == "operation") {
          tokens.push_back({Tokens::TokenType::operation, line, buffer});
        } else if (buffer == "cast") {
          tokens.push_back({Tokens::TokenType::cast, line, buffer});
        } else if (buffer == "autocast") {
          tokens.push_back({Tokens::TokenType::autocast, line, buffer});
        } else if (buffer == "define") {
          tokens.push_back({Tokens::TokenType::define, line, buffer});
        } else if (buffer == "ifdef") {
          tokens.push_back({Tokens::TokenType::ifdef, line, buffer});
        } else if (buffer == "ifndef") {
          tokens.push_back({Tokens::TokenType::ifndef, line, buffer});
        } else if (buffer == "undef") {
          tokens.push_back({Tokens::TokenType::undef, line, buffer});
        } else if (buffer == "keyword") {
          tokens.push_back({Tokens::TokenType::keyword, line, buffer});
        } else if (buffer == "macro") {
          tokens.push_back({Tokens::TokenType::macro, line, buffer});
        } else if (buffer == "endif") {
          tokens.push_back({Tokens::TokenType::endif, line, buffer});
        } else if (buffer == "logi") {
          tokens.push_back({Tokens::TokenType::logi, line, buffer});
        } else if (buffer == "logw") {
          tokens.push_back({Tokens::TokenType::logw, line, buffer});
        } else if (buffer == "loge") {
          tokens.push_back({Tokens::TokenType::loge, line, buffer});
        } else if (buffer == "template") {
          tokens.push_back({Tokens::TokenType::_template, line, buffer});
        } else if (buffer == "asm") {
          while (tryconsume(' ') || tryconsume('\r'));
          while (tryconsume('\n')) {line++;};
          if (!tryconsume('{'))
            Errors::error("Missing token", "Opening curly bracket expected", line);
          stringstream ss;
          while (hasPeek() && !tryconsume('}')) {
            ss << consume();
          }
          if (!hasPeek())
            Errors::error("Missing token", "Closing curly bracket expected", line);
          tokens.push_back({Tokens::TokenType::Asm, line, ss.str()});
        } else {
          tokens.push_back({Tokens::TokenType::identifier, line, buf.str()});
        }
        buf.str("");
      } else if (isdigit(peek())) {
        while (isdigit(peek()) || peek() == '.' || StringUtils::isInString(peek(), Constants::HEX_LETTERS))
          buf << consume();
        if (StringUtils::isInString(peek(), Constants::LITERAL_PREFIXES))
          buf << consume();
        tokens.push_back({Tokens::TokenType::literal, line, buf.str()});
        buf.str("");
      } else if (!isspace(peek())) {
        while (!isalnum(peek()) && !isspace(peek()) && !StringUtils::isInString(peek(), Constants::NON_SYMBOLS_TOKEN_CHARS))
          buf << consume();
        tokens.push_back({Tokens::TokenType::symbols, line, buf.str()});
        buf.str("");
      } else {
        Errors::error("Invalid Token", Formatting::format("Token '%s' not recognized", string(1, peek())), line);
      }
    }
  }
  this->output = tokens;
  return tokens;
}

char Tokenizer::Tokenizer::null() {
  return 0;
}

int Tokenizer::Tokenizer::getCurrentLine() {
  return this->line;
}

std::string Tokenizer::Tokenizer::getCurrentColumn() { 
  return std::string(1, peek(-1));
};

bool Tokenizer::Tokenizer::equalCriteria(char a, char b) {
  return a == b;
}

void Tokenizer::Tokenizer::print(std::ostream &stream) {
  for (Tokens::Token token : this->output) {
    token.print(stream);
    stream << '\n';
  }
}
