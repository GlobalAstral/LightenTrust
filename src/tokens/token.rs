use std::fmt::Display;


#[derive(Debug)]
pub enum Token {
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

impl Display for Token {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    match self {
      Self::ParenthesisBlock(block) => {
        let temp: Vec<String> = block.iter().map(|t| format!("{}", t)).collect();
        write!(f, "({})", temp.join(", "))
      },
      Self::CurlyBlock(block) => {
        let temp: Vec<String> = block.iter().map(|t| format!("{}", t)).collect();
        write!(f, "{{{}}}", temp.join(", "))
      },
      Self::AngleBlock(block) => {
        let temp: Vec<String> = block.iter().map(|t| format!("{}", t)).collect();
        write!(f, "<{}>", temp.join(", "))
      },
      Self::SquareBlock(block) => {
        let temp: Vec<String> = block.iter().map(|t| format!("{}", t)).collect();
        write!(f, "[{}]", temp.join(", "))
      },
      Self::Semicolon => {
        write!(f, ";")
      },
      Self::Dot => {
        write!(f, "DOT")
      },
      Self::Comma => {
        write!(f, "COMMA")
      },
      Self::Return => {
        write!(f, "return")
      },
      Self::Asm(s) => {
        write!(f, "asm \"{}\"", s)
      },
      Self::Type => {
        write!(f, "type")
      },
      Self::If => {
        write!(f, "if")
      },
      Self::Else => {
        write!(f, "else")
      },
      Self::While => {
        write!(f, "while")
      },
      Self::Do => {
        write!(f, "do")
      },
      Self::For => {
        write!(f, "for")
      },
      Self::Namespace => {
        write!(f, "namespace")
      },
      Self::Fnc => {
        write!(f, "fnc")
      },
      Self::Inline => {
        write!(f, "inline")
      },
      Self::Identifier(s) => {
        write!(f, "{}", s)
      },
      Self::Literal(lit) => {
        write!(f, "{}", lit)
      },
      Self::Symbols(s) => {
        write!(f, "{}", s)
      },
    }
  }
}
