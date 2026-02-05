use std::{iter::Peekable, process::exit};

use crate::tokens::token::{Token, TokenKind};


pub struct Tokenizer {
  input: Peekable<std::vec::IntoIter<char>>,
  line: usize,
  output: Vec<Token>
}

impl Tokenizer {
  fn error(&self, s: &str) -> ! {
    eprintln!("Error: {} at line {}", s, self.line);
    exit(1);
  }

  pub fn new(i: &str) -> Self {
    let temp: Peekable<std::vec::IntoIter<char>> = i.chars().collect::<Vec<_>>().into_iter().peekable();
    Self { input: temp, line: 1, output: Vec::new() }
  }

  fn tokenize_until(&mut self, find: char) -> Vec<Token> {
    let mut v: Vec<Token> = Vec::new();
    let mut flag = false;
    while let Some(p) = self.input.peek() {
      if *p == '\n' {
        self.line += 1;
        self.input.next();
        continue;
      }
      if *p == find {
        flag = true;
        self.input.next();
        break;
      }
      if let Some(tok) = self.token_one() {
        v.push(tok);
      }
    };
    if !flag {
      self.error(&format!("Expected '{}' closing delimiter.", find));
    }
    v
  }
  #[allow(dead_code)]
  fn get_until(&mut self, find: char) -> String {
    let mut v: String = String::new();
    let mut flag = false;
    while let Some(p) = self.input.peek() {
      if *p == find {
        flag = true;
        self.input.next();
        break;
      }
      v.push(self.input.next().unwrap());
    };
    if !flag {
      self.error(&format!("Expected '{}' closing delimiter.", find));
    }
    v
  }

  fn parse_escaped_char(&mut self) -> char {
    match self.input.next() {
      Some('\\') => {
        let next = self.input.next().unwrap_or_else(|| {
          self.error("Unexpected end of input after '\\'");
        });
        match next {
          'n' => '\n',
          'r' => '\r',
          't' => '\t',
          '\\' => '\\',
          '\'' => '\'',
          '"' => '"',
          '0' => '\0',
          'x' => {
            let hi = self.input.next()
              .and_then(|c| c.to_digit(16))
              .unwrap_or_else(|| self.error("Invalid hex escape"));
            let lo = self.input.next()
              .and_then(|c| c.to_digit(16))
              .unwrap_or_else(|| self.error("Invalid hex escape"));
            std::char::from_u32((hi << 4 | lo) as u32)
              .unwrap_or_else(|| self.error("Invalid hex value"))
          }
          'u' => {
            if self.input.next() != Some('{') {
              self.error("Expected '{' after \\u");
            }
            let mut codepoint = String::new();
            while let Some(c) = self.input.next() {
              if c == '}' { break; }
              codepoint.push(c);
            }
            if codepoint.is_empty() {
              self.error("Empty unicode escape");
            }
            let val = u32::from_str_radix(&codepoint, 16)
              .unwrap_or_else(|_| self.error("Invalid unicode escape"));
            std::char::from_u32(val)
              .unwrap_or_else(|| self.error("Invalid unicode codepoint"))
          }
          other => other,
        }
      }
      Some(c) => c,
      None => self.error("Unexpected end of input"),
    }
  }

  fn is_char_present(&mut self, c: char) -> bool {
    let temp = self.input.clone();
    while let Some(ch) = self.input.peek() {
      if *ch == c {
        self.input = temp;
        return true;
      }
      self.input.next();
    };
    self.input = temp;
    return false;
  }

  fn token_one(&mut self) -> Option<Token> {
    let kind = if let Some(ch) = self.input.next() {
      match ch {
        '(' => Some(TokenKind::ParenthesisBlock(self.tokenize_until(')'))),
        '{' => Some(TokenKind::CurlyBlock(self.tokenize_until('}'))),
        '<' if self.is_char_present('>') => Some(TokenKind::AngleBlock(self.tokenize_until('>'))),
        '[' => Some(TokenKind::SquareBlock(self.tokenize_until(']'))),
        ';' => Some(TokenKind::Semicolon),
        '.' => Some(TokenKind::Dot),
        ',' => Some(TokenKind::Comma),
        '&' => Some(TokenKind::Ampersand),
        '$' => Some(TokenKind::Dollar),
        '#' => Some(TokenKind::Hash),
        '\'' => {
          if let Some(ch) = self.input.next() {
            let parsed = if ch == '\\' {
              self.parse_escaped_char()
            } else {
              ch
            };
            match self.input.next() {
              Some('\'') => Some(TokenKind::Literal(format!("'{}'", parsed))),
              _ => self.error("Expected closing single quote")
            }
          } else {
            self.error("Expected character literal")
          }
        },
        '"' => {
          let mut buf = String::new();
          while let Some(&ch) = self.input.peek() {
            if ch == '\n' {
              self.line += 1;
              self.input.next();
              continue;
            }
            if ch == '"' { self.input.next(); break; }
            buf.push(self.parse_escaped_char());
          }
          Some(TokenKind::Literal(format!("\"{}\"", buf)))
        },
        c => {
          if let Some(cha) = self.input.peek() && c == '0' && *cha == 'x' {
            let mut buf: String = String::from(c);
            buf.push(self.input.next().unwrap());
            while let Some(ch) = self.input.peek() {
              if ch.is_digit(16) {
                buf.push(self.input.next().unwrap());
                continue;
              }
              break;
            };
            Some(TokenKind::Literal(buf))
          } else if c.is_alphabetic() {
            let mut buf: String = String::from(c);
            while let Some(ch) = self.input.peek() {
              if ch.is_alphanumeric() {
                buf.push(self.input.next().unwrap());
                continue;
              }
              break;
            };
            match buf.as_str() {
              "return" => Some(TokenKind::Return),
              "asm" => Some(TokenKind::Asm),
              "typedef" => Some(TokenKind::Typedef),
              "if" => Some(TokenKind::If),
              "else" => Some(TokenKind::Else),
              "while" => Some(TokenKind::While),
              "do" => Some(TokenKind::Do),
              "for" => Some(TokenKind::For),
              "namespace" => Some(TokenKind::Namespace),
              "fnc" => Some(TokenKind::Fnc),
              "inline" => Some(TokenKind::Inline),
              "struct" => Some(TokenKind::Struct),
              "union" => Some(TokenKind::Union),
              "enum" => Some(TokenKind::Enum),
              "to" => Some(TokenKind::To),
              "sizeof" => Some(TokenKind::SizeOf),
              "operator" => Some(TokenKind::Operator),
              "mut" => Some(TokenKind::Mut),
              "break" => Some(TokenKind::Break),
              "continue" => Some(TokenKind::Continue),
              "signed" => Some(TokenKind::Signed),
              "include" => Some(TokenKind::Include),
              "define" => Some(TokenKind::Define),
              "macro" => Some(TokenKind::Macro),
              "getconfig" => Some(TokenKind::GetConfig),
              "ifdef" => Some(TokenKind::Ifdef),
              "ifndef" => Some(TokenKind::Ifndef),
              s => Some(TokenKind::Identifier(s.to_string()))
            }
          } else if c.is_digit(10) {
            let mut buf: String = String::from(c);
            while let Some(ch) = self.input.peek() {
              if ch.is_digit(10) || *ch == '.' {
                buf.push(self.input.next().unwrap());
                continue;
              }
              break;
            };
            Some(TokenKind::Literal(buf))
          } else if !c.is_whitespace() && !c.is_alphanumeric() {
            let mut buf: String = String::from(c);
            while let Some(ch) = self.input.peek() {
              if !ch.is_whitespace() && !ch.is_alphanumeric() && (*ch != '<' || !self.is_char_present('>')) {
                buf.push(self.input.next().unwrap());
                continue;
              }
              break;
            };
            Some(TokenKind::Symbols(buf))
          } else {
            None
          }
        }
      }
    } else {
      None
    };
    if let Some(kind) = kind {
      Some(Token {kind: kind, line: self.line})
    } else {
      None
    }
  }

  pub fn tokenize(&mut self) -> &Vec<Token> {
    while let Some(ch) = self.input.peek() {
      if *ch == '\n' {
        self.line += 1;
        self.input.next();
        continue;
      }
      if ch.is_whitespace() {
        self.input.next();
        continue;
      }

      if let Some(token) = self.token_one() {
        self.output.push(token);
      } else {
        self.error("Unexpected Token");
      }
    }
    &self.output
  }
}
