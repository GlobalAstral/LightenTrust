use crate::parser::{literals::Literal, types::Type};

#[derive(Debug, PartialEq, PartialOrd, Clone)]
pub enum ExprKind {
  Literal(Literal),
  Variable(u64),
  FncCall {
    id: u64,
    args: Vec<Expression>
  },
  FncPtrCall {
    id: u64,
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
    field: String
  },
  Cast {
    base: Box<Expression>,
    into: Type
  },
  FncPtrRef(u64),
  SizeOf(Box<Expression>),
  //TODO ADD CUSTOM OPERANDS LATER!
}

#[derive(Debug, PartialEq, PartialOrd, Clone)]
pub struct Expression {
  pub kind: ExprKind,
  pub return_type: Type,
}
