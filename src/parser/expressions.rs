use crate::parser::{literals::Literal, types::Type};

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

pub struct Expression {
  kind: ExprKind,
  return_type: Type,
}
