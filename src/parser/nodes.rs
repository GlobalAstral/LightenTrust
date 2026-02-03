use std::fmt::Display;

use crate::parser::{expressions::{Expression, Operator}, types::{Type, Variable}};

#[derive(Debug, PartialEq, PartialOrd, Clone)]
pub struct Fnc {
  pub return_type: Box<Type>,
  pub name: String,
  pub arguments: Vec<Variable>,
  pub body: Option<Box<Node>>,
  pub id: u64,
}

impl Display for Fnc  {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    write!(f, "fnc {}<{}>({}) {}", self.name, self.id, self.arguments.iter().map(|a| format!("{}", a)).collect::<Vec<String>>().join(", "), self.return_type)
  }
}

#[derive(Debug, PartialEq, PartialOrd, Clone, Default)]
pub enum Node {
  Scope(Vec<Node>),
  FncDecl(Fnc),
  OperatorDecl(Operator),

  Expr(Expression),
  #[default]
  Invalid
}

impl Display for Node {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    match self {
      Self::Scope(s) => write!(f, "{{\n\t{}}}", s.iter().map(|n| format!("{}", n)).collect::<Vec<String>>().join("\n\t")),
      Self::FncDecl(fnc) => write!(f, "fnc {}<{}>({}) {} {}", fnc.name, fnc.id, fnc.arguments.iter().map(|a| format!("{}", a)).collect::<Vec<String>>().join(", "), fnc.return_type, if let Some(p) = &fnc.body {format!("{}", p)} else {String::from(";")}),
      Self::Expr(e) => write!(f, "{}", e),
      Self::OperatorDecl(operator) => {
        let temp = if operator.right.is_some() {
          format!("{}", operator.right.as_ref().unwrap())
        } else { String::new() };
        write!(f, "{} {} {} - {} -> {}", operator.left, operator.symbols, temp, operator.precedence, operator.return_type)
      },
      Self::Invalid => write!(f, "NULL"),
    }
  }
}
