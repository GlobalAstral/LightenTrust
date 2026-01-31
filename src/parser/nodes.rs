use std::fmt::Display;

use crate::parser::types::{Type, Variable};

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone, Hash)]
pub struct Fnc {
  pub return_type: Box<Type>,
  pub name: String,
  pub arguments: Vec<Variable>,
  pub body: Box<Node>,
  pub id: u64,
}

impl Display for Fnc  {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    write!(f, "fnc {}<{}>({}) {}", self.name, self.id, self.arguments.iter().map(|a| format!("{}", a)).collect::<Vec<String>>().join(", "), self.return_type)
  }
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone, Hash, Default)]
pub enum Node {
  Scope(Vec<Node>),
  FncDecl(Fnc),

  #[default]
  Invalid
}

impl Display for Node {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    match self {
      Self::Scope(s) => write!(f, "{{\n\t{}}}", s.iter().map(|n| format!("{}", n)).collect::<Vec<String>>().join("\n\t")),
      Self::FncDecl(fnc) => write!(f, "fnc {}<{}>({}) {} {}", fnc.name, fnc.id, fnc.arguments.iter().map(|a| format!("{}", a)).collect::<Vec<String>>().join(", "), fnc.return_type, fnc.body),
      Self::Invalid => write!(f, "NULL")
    }
  }
}
