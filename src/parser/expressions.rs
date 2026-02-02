use std::fmt::Display;

use crate::parser::{literals::Literal, types::{Type, Variable}};

#[derive(Debug, PartialEq, PartialOrd, Clone)]
pub enum ExprKind {
  Literal(Literal),
  Variable(u64),
  FncCall {
    id: u64,
    args: Vec<Expression>
  },
  FncPtrCall {
    expr: Box<Expression>,
    args: Vec<Expression>
  },
  Reference(Box<Expression>),
  Dereference(Box<Expression>),
  Index {
    base: Box<Expression>,
    index: Box<Expression>
  },
  FieldAccess {
    base: Box<Expression>,
    field: Variable
  },
  Cast {
    base: Box<Expression>,
    into: Type
  },
  FncPtrRef(u64),
  SizeOf(Box<Expression>),
  //TODO ADD CUSTOM OPERANDS LATER!
}

impl Display for ExprKind {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    match self {
      Self::Cast { base, into } => write!(f, "{} to {}", base, into),
      Self::Dereference(e) => write!(f, "*{}", e),
      Self::FieldAccess { base, field } => write!(f, "{}.{}", base, field),
      Self::FncCall { id, args } => write!(f, "fnc<{}>({})", id, args.iter().map(|a| format!("{}", a)).collect::<Vec<String>>().join(", ")),
      Self::FncPtrCall { expr, args } => write!(f, "{}({})", expr, args.iter().map(|a| format!("{}", a)).collect::<Vec<String>>().join(", ")),
      Self::FncPtrRef(id) => write!(f, "(&fnc<{}>)", id),
      Self::Index { base, index } => write!(f, "{}[{}]", base, index),
      Self::Literal(l) => write!(f, "{}", l),
      Self::Reference(r) => write!(f, "&{}", r),
      Self::SizeOf(e) => write!(f, "sizeof {}", e),
      Self::Variable(v) => write!(f, "({})", v) 
    }
  }
}

impl Display for Expression {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    write!(f, "{} -> {}", self.kind, self.return_type)
  }
}

#[derive(Debug, PartialEq, PartialOrd, Clone)]
pub struct Expression {
  pub kind: ExprKind,
  pub return_type: Type,
}
