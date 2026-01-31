use std::fmt::Display;

use crate::parser::literals::Literal;

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone, Hash, Default)]
pub enum TokenKind {
  ParenthesisBlock(Vec<Token>),
  CurlyBlock(Vec<Token>),
  AngleBlock(Vec<Token>),
  SquareBlock(Vec<Token>),
  Semicolon, Dot, Comma, Ampersand,
  Return, Asm(String), Type, If, Else, While, Do, For, Namespace, Fnc, Inline, Struct, Union, Enum,
  Identifier(String),
  Literal(String),
  Symbols(String),
  #[default]
  Invalid
}
#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone, Hash, Default)]
pub struct Token {
  pub kind: TokenKind,
  pub line: usize
}

impl Token {
  pub fn as_square_block(&self) -> Option<Vec<Token>> {
    match &self.kind {
      TokenKind::SquareBlock(s) => Some(s.clone()),
      _ => None
    }
  }
  pub fn as_curly_block(&self) -> Option<Vec<Token>> {
    match &self.kind {
      TokenKind::CurlyBlock(s) => Some(s.clone()),
      _ => None
    }
  }
  pub fn as_paren_block(&self) -> Option<Vec<Token>> {
    match &self.kind {
      TokenKind::ParenthesisBlock(s) => Some(s.clone()),
      _ => None
    }
  }
  pub fn as_angle_block(&self) -> Option<Vec<Token>> {
    match &self.kind {
      TokenKind::AngleBlock(s) => Some(s.clone()),
      _ => None
    }
  }
  pub fn as_asm(&self) -> Option<&str> {
    match &self.kind {
      TokenKind::Asm(s) => Some(&s),
      _ => None
    }
  }
  pub fn as_identifier(&self) -> Option<&str> {
    match &self.kind {
      TokenKind::Identifier(s) => Some(s),
      _ => None
    }
  }
  pub fn is_identifier_and(&self, f: &dyn Fn(&str) -> bool) -> bool {
    match &self.kind {
      TokenKind::Identifier(s) if f(s) => true,
      _ => false
    }
  }
  pub fn as_symbols(&self) -> Option<&str> {
    match &self.kind {
      TokenKind::Symbols(s) => Some(s),
      _ => None
    }
  }
  pub fn is_symbols_of(&self, sym: &str) -> bool {
    match &self.kind {
      TokenKind::Symbols(s) if s == sym => true,
      _ => false
    }
  }
  pub fn as_literal(&self) -> Option<Literal> {
    match &self.kind {
      TokenKind::Literal(s) => {
        match Literal::from(s) {
          Ok(l) => Some(l),
          _ => None
        }
      },
      _ => None
    }
  }
}

impl Display for Token {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    match &self.kind {
      TokenKind::ParenthesisBlock(block) => {
        let temp: Vec<String> = block.iter().map(|t| format!("{}", t)).collect();
        write!(f, "({})", temp.join(", "))
      },
      TokenKind::CurlyBlock(block) => {
        let temp: Vec<String> = block.iter().map(|t| format!("{}", t)).collect();
        write!(f, "{{{}}}", temp.join(", "))
      },
      TokenKind::AngleBlock(block) => {
        let temp: Vec<String> = block.iter().map(|t| format!("{}", t)).collect();
        write!(f, "<{}>", temp.join(", "))
      },
      TokenKind::SquareBlock(block) => {
        let temp: Vec<String> = block.iter().map(|t| format!("{}", t)).collect();
        write!(f, "[{}]", temp.join(", "))
      },
      TokenKind::Semicolon => {
        write!(f, ";")
      },
      TokenKind::Dot => {
        write!(f, "DOT")
      },
      TokenKind::Comma => {
        write!(f, "COMMA")
      },
      TokenKind::Ampersand => {
        write!(f, "AMPERSAND")
      }
      TokenKind::Return => {
        write!(f, "return")
      },
      TokenKind::Asm(s) => {
        write!(f, "asm \"{}\"", s)
      },
      TokenKind::Type => {
        write!(f, "type")
      },
      TokenKind::If => {
        write!(f, "if")
      },
      TokenKind::Else => {
        write!(f, "else")
      },
      TokenKind::While => {
        write!(f, "while")
      },
      TokenKind::Do => {
        write!(f, "do")
      },
      TokenKind::For => {
        write!(f, "for")
      },
      TokenKind::Namespace => {
        write!(f, "namespace")
      },
      TokenKind::Fnc => {
        write!(f, "fnc")
      },
      TokenKind::Inline => {
        write!(f, "inline")
      },
      TokenKind::Struct => {
        write!(f, "struct")
      },
      TokenKind::Union => {
        write!(f, "union")
      },
      TokenKind::Enum => {
        write!(f, "enum")
      },
      TokenKind::Identifier(s) => {
        write!(f, "{}", s)
      },
      TokenKind::Literal(lit) => {
        write!(f, "{}", lit)
      },
      TokenKind::Symbols(s) => {
        write!(f, "{}", s)
      },
      TokenKind::Invalid => {
        write!(f, "NULL")
      }
    }?;
    write!(f, "<{}>", self.line)
  }
}
