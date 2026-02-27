use std::fmt::Display;

use crate::parser::{literals::Literal, nodes::Node, types::{Type, Variable}};

#[derive(Debug, PartialEq, PartialOrd, Clone)]
pub struct Operator {
  pub symbols: String,
  pub left: Variable,
  pub right: Option<Variable>,
  pub return_type: Type,
  pub precedence: u64,
  pub body: Box<Node>
}

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
  Unary {
    expr: Box<Expression>,
    operator: Operator
  },
  Binary {
    left: Box<Expression>,
    right: Box<Expression>,
    operator: Operator
  },
  Assignment {
    left: Box<Expression>,
    right: Box<Expression>
  }
}

impl Display for ExprKind {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    match self {
      Self::Cast { base, into } => write!(f, "{} to {}", base, into),
      Self::Dereference(e) => write!(f, "*({})", e),
      Self::FieldAccess { base, field } => write!(f, "{}.{}", base, field),
      Self::FncCall { id, args } => write!(f, "fnc<{}>({})", id, args.iter().map(|a| format!("{}", a)).collect::<Vec<String>>().join(", ")),
      Self::FncPtrCall { expr, args } => write!(f, "{}({})", expr, args.iter().map(|a| format!("{}", a)).collect::<Vec<String>>().join(", ")),
      Self::FncPtrRef(id) => write!(f, "(&fnc<{}>)", id),
      Self::Index { base, index } => write!(f, "{}[{}]", base, index),
      Self::Literal(l) => write!(f, "{}", l),
      Self::Reference(r) => write!(f, "&{}", r),
      Self::SizeOf(e) => write!(f, "sizeof {}", e),
      Self::Variable(v) => write!(f, "({})", v),
      Self::Unary { expr, operator } => write!(f, "{}{}", operator.symbols, expr),
      Self::Binary { left, right, operator } => write!(f, "{} {} {}", left, operator.symbols, right),
      Self::Assignment { left, right } => write!(f, "{} = {}", left, right),
    }
  }
}

impl Display for Expression {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    write!(f, "( {} -> {} )", self.kind, self.return_type)
  }
}

impl Expression {
  pub fn is_evaluable(&self, vars: &Vec<Variable>) -> bool {
    match &self.kind {
      ExprKind::Literal(_) => true,
      ExprKind::Cast { base, .. } => base.is_evaluable(vars),
      ExprKind::Variable(id) => {
        vars.iter().find(|v| &v.id == id).is_some_and(|temp| temp.global)
      },
      ExprKind::SizeOf(_) => true,
      _ => false
    }
  }
}

#[derive(Debug, PartialEq, PartialOrd, Clone)]
pub struct Expression {
  pub kind: ExprKind,
  pub return_type: Type,
}
