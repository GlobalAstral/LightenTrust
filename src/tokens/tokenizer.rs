use std::{path::PathBuf, process::exit};

use crate::{parser::utils::Processor, tokens::token::{Token, TokenKind}};


pub struct Tokenizer {
  base: Processor<char>,
  line: usize,
  output: Vec<Token>,
  comment: bool,
  multicomment: bool,
  file: PathBuf,
}

impl Tokenizer {
  fn error(&self, s: &str) -> ! {
    eprintln!("Error: {} at line: {} in file: {}", s, self.line, self.file.display());
    exit(1);
  }

  pub fn new(i: &str, file: PathBuf) -> Self {
    Self { base: Processor::new(i.chars().collect::<Vec<char>>(), Box::new(|a, b| a == b), Box::new(|_| 0), Box::new(|_| PathBuf::new())), line: 1, output: Vec::new(), comment: false, multicomment: false, file: file }
  }

  fn tokenize_until(&mut self, find: char) -> Vec<Token> {
    let mut v: Vec<Token> = Vec::new();
    let mut flag = false;
    while self.base.has_peek() {
      if self.process_comments() {
        continue;
      }
      if self.base.peek() == find {
        flag = true;
        self.base.consume();
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
    while self.base.has_peek() {
      if self.base.peek() == find {
        flag = true;
        self.base.consume();
        break;
      }
      v.push(self.base.consume());
    };
    if !flag {
      self.error(&format!("Expected '{}' closing delimiter.", find));
    }
    v
  }

  fn parse_escaped_char(&mut self) -> char {
    match self.base.consume() {
      '\\' => {
        let next = self.base.consume();
        match next {
          'n' => '\n',
          'r' => '\r',
          't' => '\t',
          '\\' => '\\',
          '\'' => '\'',
          '"' => '"',
          '0' => '\0',
          'x' => {
            let hi = self.base.consume().to_digit(16)
              .unwrap_or_else(|| self.error("Invalid hex escape"));
            let lo = self.base.consume().to_digit(16)
              .unwrap_or_else(|| self.error("Invalid hex escape"));
            std::char::from_u32((hi << 4 | lo) as u32)
              .unwrap_or_else(|| self.error("Invalid hex value"))
          }
          'u' => {
            if self.base.consume() != '{' {
              self.error("Expected '{' after \\u");
            }
            let mut codepoint = String::new();
            while self.base.has_peek() {
              if self.base.peek() == '}' { break; }
              codepoint.push(self.base.consume());
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
      c => c,
    }
  }

  fn is_char_present(&mut self, c: char) -> bool {
    let revert = self.base.peek;
    while self.base.has_peek() {
      if self.base.peek() == c {
        self.base.peek = revert;
        return true;
      }
      self.base.consume();
    };
    self.base.peek = revert;
    return false;
  }

  fn token_one(&mut self) -> Option<Token> {
    let kind =  match self.base.consume() {
        '(' => Some(TokenKind::ParenthesisBlock(self.tokenize_until(')'))),
        '{' => Some(TokenKind::CurlyBlock(self.tokenize_until('}'))),
        '<' if self.is_char_present('>') => Some(TokenKind::AngleBlock(self.tokenize_until('>'))),
        '[' => Some(TokenKind::SquareBlock(self.tokenize_until(']'))),
        ';' => Some(TokenKind::Semicolon),
        '.' => {
          if self.base.tryconsume('.') {
            if self.base.tryconsume('.') {
              Some(TokenKind::Ellipsis)
            } else {
              Some(TokenKind::TwoDots)
            }
          } else {
            Some(TokenKind::Dot)
          }
        },
        ',' => Some(TokenKind::Comma),
        '&' => Some(TokenKind::Ampersand),
        '$' => Some(TokenKind::Dollar),
        '#' => Some(TokenKind::Hash),
        '\'' => {
          let parsed = if self.base.peek() == '\\' {
            self.parse_escaped_char()
          } else {
            self.base.consume()
          };
          match self.base.consume() {
            '\'' => Some(TokenKind::Literal(format!("'{}'", parsed))),
            _ => self.error("Expected closing single quote")
          }
        },
        '"' => {
          let mut buf = String::new();
          while self.base.has_peek() {
            if self.process_comments() {
              continue;
            }
            if self.base.peek() == '"' { self.base.consume(); break; }
            buf.push(self.parse_escaped_char());
          }
          Some(TokenKind::Literal(format!("\"{}\"", buf)))
        },
        c => {
          if c == '0' && self.base.peek() == 'x' {
            let mut buf: String = String::from(c);
            buf.push(self.base.consume());
            while self.base.has_peek() {
              if self.base.peek().is_digit(16) {
                buf.push(self.base.consume());
                continue;
              }
              break;
            };
            Some(TokenKind::Literal(buf))
          } else if c.is_alphabetic() {
            let mut buf: String = String::from(c);
            while self.base.has_peek() {
              if self.base.peek().is_alphanumeric() {
                buf.push(self.base.consume());
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
              "extern" => Some(TokenKind::Extern),
              s => Some(TokenKind::Identifier(s.to_string()))
            }
          } else if c.is_digit(10) {
            let mut buf: String = String::from(c);
            while self.base.has_peek() {
              if self.base.peek().is_digit(10) || self.base.peek() == '.' {
                buf.push(self.base.consume());
                continue;
              }
              break;
            };
            Some(TokenKind::Literal(buf))
          } else if !c.is_whitespace() && !c.is_alphanumeric() {
            let mut buf: String = String::from(c);
            while self.base.has_peek() {
              if !self.base.peek().is_whitespace() && !self.base.peek().is_alphanumeric() && (self.base.peek() != '<' || !self.is_char_present('>')) {
                buf.push(self.base.consume());
                continue;
              }
              break;
            };
            Some(TokenKind::Symbols(buf))
          } else {
            None
          }
        }
      };
    if let Some(kind) = kind {
      Some(Token {kind: kind, line: self.line, file: self.file.clone()})
    } else {
      None
    }
  }

  fn process_comments(&mut self) -> bool {
    let revert = self.base.peek;
    if self.base.tryconsume('\n') {
      self.line += 1;
      self.comment = false;
      return true;
    } else if self.base.tryconsume('/') {
      if self.base.tryconsume('/') {
        self.comment = true;
        return true;
      } else if self.base.tryconsume('*') {
        self.multicomment = true;
        return true;
      } else {
        self.base.peek = revert;
      }
    } else if self.multicomment &&  self.base.tryconsume('*') {
      if self.base.tryconsume('/') {
        self.multicomment = false;
        return true;
      } else {
        self.base.peek = revert;
      }
    } else if self.comment || self.multicomment {
      self.base.consume();
      return true;
    }
    false
  }

  pub fn tokenize(&mut self) -> &Vec<Token> {
    while self.base.has_peek() {
      if self.process_comments() {
        continue;
      }
      if self.base.peek().is_whitespace() {
        self.base.consume();
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
