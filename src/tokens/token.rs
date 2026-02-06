use std::{fmt::Display, path::PathBuf};

use crate::parser::literals::Literal;

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone, Hash, Default)]
pub enum TokenKind {
  ParenthesisBlock(Vec<Token>),
  CurlyBlock(Vec<Token>),
  AngleBlock(Vec<Token>),
  SquareBlock(Vec<Token>),
  Semicolon, Dot, Comma, Ampersand, Dollar, Hash, Ellipsis, TwoDots,
  Return, Asm, Typedef, If, Else, While, Do, For, Namespace, Fnc, Inline, Struct, Union, Enum, To, SizeOf, Operator, Mut, Break, Continue, Signed, Extern,
  Include, Define, Macro, GetConfig, Ifdef, Ifndef,
  Identifier(String),
  Literal(String),
  Symbols(String),
  #[default]
  Invalid
}
#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone, Hash, Default)]
pub struct Token {
  pub kind: TokenKind,
  pub line: usize,
  pub file: PathBuf
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
  pub fn as_identifier(&self) -> Option<&str> {
    match &self.kind {
      TokenKind::Identifier(s) => Some(s),
      _ => None
    }
  }
  pub fn as_symbols(&self) -> Option<&str> {
    match &self.kind {
      TokenKind::Symbols(s) => Some(s),
      _ => None
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
      TokenKind::Semicolon => write!(f, ";"),
      TokenKind::Dot => write!(f, "DOT"),
      TokenKind::TwoDots => write!(f, "TWODOTS"),
      TokenKind::Comma => write!(f, "COMMA"),
      TokenKind::Ampersand => write!(f, "AMPERSAND"),
      TokenKind::Dollar => write!(f, "DOLLAR"),
      TokenKind::Ellipsis => write!(f, "ELLIPSIS"),
      TokenKind::Hash => write!(f, "HASH"),
      TokenKind::Return => write!(f, "return"),
      TokenKind::Extern => write!(f, "extern"),
      TokenKind::Signed => write!(f, "unsigned"),
      TokenKind::Break => write!(f, "break"),
      TokenKind::Continue => write!(f, "continue"),
      TokenKind::SizeOf => write!(f, "sizeof"),
      TokenKind::Operator => write!(f, "operator"),
      TokenKind::Asm=> write!(f, "asm"),
      TokenKind::Typedef => write!(f, "typedef"),
      TokenKind::If => write!(f, "if"),
      TokenKind::Else => write!(f, "else"),
      TokenKind::While => write!(f, "while"),
      TokenKind::Do => write!(f, "do"),
      TokenKind::Mut => write!(f, "mut"),
      TokenKind::For => write!(f, "for"),
      TokenKind::Namespace => write!(f, "namespace"),
      TokenKind::Fnc => write!(f, "fnc"),
      TokenKind::Inline => write!(f, "inline"),
      TokenKind::Struct => write!(f, "struct"),
      TokenKind::Union => write!(f, "union"),
      TokenKind::Enum => write!(f, "enum"),
      TokenKind::To => write!(f, "to"),
      TokenKind::Identifier(s) => write!(f, "{}", s),
      TokenKind::Literal(lit) => write!(f, "{}", lit),
      TokenKind::Symbols(s) => write!(f, "{}", s),
      TokenKind::Include => write!(f, "include"),
      TokenKind::Define => write!(f, "define"),
      TokenKind::Macro => write!(f, "macro"),
      TokenKind::GetConfig => write!(f, "getconfig"),
      TokenKind::Ifdef => write!(f, "ifdef"),
      TokenKind::Ifndef => write!(f, "ifndef"),
      TokenKind::Invalid => write!(f, "NULL")
    }?;
    write!(f, "<{}>", self.line)
  }
}
