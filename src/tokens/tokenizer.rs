use std::{iter::Peekable, process::exit};

use crate::tokens::token::{Token};


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

  fn token_one(&mut self) -> Option<Token> {
    if let Some(ch) = self.input.next() {
      match ch {
        '(' => Some(Token::ParenthesisBlock(self.tokenize_until(')'))),
        '{' => Some(Token::CurlyBlock(self.tokenize_until('}'))),
        '<' => Some(Token::AngleBlock(self.tokenize_until('>'))),
        '[' => Some(Token::SquareBlock(self.tokenize_until(']'))),
        ';' => Some(Token::Semicolon),
        '.' => Some(Token::Dot),
        ',' => Some(Token::Comma),
        '\'' => {
          if let Some(ch) = self.input.next() {
            let parsed = if ch == '\\' {
              self.parse_escaped_char()
            } else {
              ch
            };
            match self.input.next() {
              Some('\'') => Some(Token::Literal(format!("'{}'", parsed))),
              _ => self.error("Expected closing single quote")
            }
          } else {
            self.error("Expected character literal")
          }
        },
        '"' => {
          let mut buf = String::new();
          while let Some(&ch) = self.input.peek() {
            if ch == '"' { self.input.next(); break; }
            buf.push(self.parse_escaped_char());
          }
          Some(Token::Literal(format!("\"{}\"", buf)))
        },
        c => {
          if let Some(cha) = self.input.peek() && c == '0' && *cha == 'x' {
            let mut buf: String = String::from(c);
            buf.push(self.input.next().unwrap());
            while let Some(ch) = self.input.next() {
              if ch.is_digit(16) {
                buf.push(ch);
                continue;
              }
              break;
            };
            Some(Token::Literal(buf))
          } else if c.is_alphabetic() {
            let mut buf: String = String::from(c);
            while let Some(ch) = self.input.next() {
              if ch.is_alphanumeric() {
                buf.push(ch);
                continue;
              }
              break;
            };
            match buf.as_str() {
              "return" => Some(Token::Return),
              "asm" => {
                if let Some(ch) = self.input.peek() && *ch != '{' {
                  self.error("Expected '{' after 'asm'");
                }
                self.input.next();
                Some(Token::Asm(self.get_until('}')))
              },
              "type" => Some(Token::Type),
              "if" => Some(Token::If),
              "else" => Some(Token::Else),
              "while" => Some(Token::While),
              "do" => Some(Token::Do),
              "for" => Some(Token::For),
              "namespace" => Some(Token::Namespace),
              "fnc" => Some(Token::Fnc),
              "inline" => Some(Token::Inline),
              s => Some(Token::Identifier(s.to_string()))
            }
          } else if c.is_digit(10) {
            let mut buf: String = String::from(c);
            while let Some(ch) = self.input.next() {
              if ch.is_digit(10) || ch == '.' {
                buf.push(ch);
                continue;
              }
              break;
            };
            Some(Token::Literal(buf))
          } else if !c.is_whitespace() {
            let mut buf: String = String::from(c);
            while let Some(ch) = self.input.next() {
              if !ch.is_whitespace() {
                buf.push(ch);
                continue;
              }
              break;
            };
            Some(Token::Symbols(buf))
          } else {
            None
          }
        }
      }
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
