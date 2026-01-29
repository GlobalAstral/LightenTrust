use std::fmt::Display;


pub enum TokenKind {
  ParenthesisBlock(Vec<Token>),
  CurlyBlock(Vec<Token>),
  AngleBlock(Vec<Token>),
  SquareBlock(Vec<Token>),
  Semicolon, Dot, Comma,
  Return, Asm(String), Type, If, Else, While, Do, For, Namespace, Fnc, Inline,
  Identifier(String),
  Literal(String),
  Symbols(String)
}
pub struct Token {
  pub kind: TokenKind,
  pub line: usize
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
      TokenKind::Identifier(s) => {
        write!(f, "{}", s)
      },
      TokenKind::Literal(lit) => {
        write!(f, "{}", lit)
      },
      TokenKind::Symbols(s) => {
        write!(f, "{}", s)
      },
    }?;
    write!(f, "<{}>", self.line)
  }
}
