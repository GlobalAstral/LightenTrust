use std::fmt::Display;

use crate::parser::{assembly::AssemblyChunk, expressions::{Expression, Operator}, types::{Type, Variable}, utils::ABI};



#[derive(Debug, PartialEq, PartialOrd, Clone)]
pub struct Linkage {
  abi: ABI,
}

impl Display for Linkage  {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    write!(f, "{:?}", self.abi)
  }
}

#[derive(Debug, PartialEq, PartialOrd, Clone)]
pub struct Fnc {
  pub return_type: Box<Type>,
  pub name: String,
  pub arguments: Vec<Variable>,
  pub body: Option<Box<Node>>,
  pub id: u64,
  pub variadic: bool,
  pub linkage: Option<Linkage>
}

impl Display for Fnc  {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    let temp = if self.linkage.is_some() {
      &format!("{}", self.linkage.as_ref().unwrap())
    } else { "default" };
    write!(f, "fnc [{}, variadic:{}] {}<{}>({}) {}", temp, self.variadic,  self.name, self.id, self.arguments.iter().map(|a| format!("{}", a)).collect::<Vec<String>>().join(", "), self.return_type)
  }
}

#[derive(Debug, PartialEq, PartialOrd, Clone, Default)]
pub enum Node {
  Scope(Vec<Node>),
  Packet(Vec<Node>),
  FncDecl(Fnc),
  OperatorDecl(Operator),
  VariableDecl {
    var: Variable,
    expr: Option<Expression>
  },
  VariableSet {
    var: Variable,
    expr: Expression
  },
  Return(Expression),
  Assembly(Vec<AssemblyChunk>),
  If(Expression, Box<Node>, Option<Box<Node>>),
  While(Expression, Box<Node>),
  DoWhile(Expression, Box<Node>),
  For {
    var: Variable,
    init: Expression,
    cond: Expression,
    incr: Box<Node>,
    body: Box<Node>
  },
  Continue,
  Break,
  Expr(Expression),

  Ignored,
  #[default]
  Invalid
}

impl Display for Node {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    match self {
      Self::Scope(s) => write!(f, "{{\n\t{}}}", s.iter().map(|n| format!("{}", n)).collect::<Vec<String>>().join("\n\t")),
      Self::Packet(s) => write!(f, "[\n\t{}]", s.iter().map(|n| format!("{}", n)).collect::<Vec<String>>().join("\n\t")),
      Self::FncDecl(fnc) => write!(f, "{}", fnc),
      Self::Expr(e) => write!(f, "{}", e),
      Self::OperatorDecl(operator) => {
        let temp = if operator.right.is_some() {
          format!("{}", operator.right.as_ref().unwrap())
        } else { String::new() };
        write!(f, "{} {} {} - {} -> {}", operator.left, operator.symbols, temp, operator.precedence, operator.return_type)
      },
      Self::VariableDecl { var, expr } => write!(f, "{} {}", var, if expr.is_some() {format!("= {}", expr.as_ref().unwrap())} else {String::new()}),
      Self::VariableSet { var, expr } => write!(f, "{} = {}", var, expr),
      Self::Return(e) => write!(f, "return {}", e),
      Self::If(c, b, e) => write!(f, "if {} {} {}", c, b, if let Some(e) = e {format!("else {}", e)} else {String::new()}),
      Self::While(c, b) => write!(f, "while {} {}", c, b),
      Self::DoWhile(c, b) => write!(f, "do {} while {}", b, c),
      Self::For { var, init, cond, incr, body } => write!(f, "for ({} = {}; {}; {}) {}", var, init, cond, incr, body),
      Self::Ignored => write!(f, "Ignored"),
      Self::Invalid => write!(f, "NULL"),
      Self::Assembly(code) => write!(f, "{:?}", code),
      Self::Continue => write!(f, "continue"),
      Self::Break => write!(f, "break"),
    }
  }
}
