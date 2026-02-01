use std::error::Error;

use crate::parser::types::{MemoryKind, Type};

#[derive(Debug, PartialEq, PartialOrd, Clone)]
pub enum Literal {
  Integer(u64),
  Float(f64),
  Char(u8),
  String(String)
}

impl Literal {
  pub fn from(s: &str) -> Result<Self, Box<dyn Error>> {
    if s.starts_with('\'') {
      Ok(Literal::Char(s.bytes().nth(1).unwrap()))
    } else if s.starts_with('"') {
      Ok(Literal::String(s[1..s.len()-1].to_string()))
    } else if let Some(hex) = s.strip_prefix("0x") {
      let num = u64::from_str_radix(hex, 16)?;
      Ok(Literal::Integer(num))
    } else if s.chars().filter(|&x| x == '.').count() == 1 {
      let num = s.parse::<f64>()?;
      Ok(Literal::Float(num))
    } else if s.chars().filter(|&x| x == '.').count() == 0 {
      let num = s.parse::<u64>()?;
      Ok(Literal::Integer(num))
    } else {
      Err(format!("Invalid literal \"{}\"", s).into())
    }
  }

  pub fn get_type(&self) -> Type {
    match self {
      Self::Integer(_) => Type::Memory { size: 4, kind: MemoryKind::Integer },
      Self::Float(_) => Type::Memory { size: 4, kind: MemoryKind::Float },
      Self::String(s) => Type::Memory { size: s.len() as u64, kind: MemoryKind::Integer },
      Self::Char(_) => Type::Memory { size: 1, kind: MemoryKind::Integer }
    }
  }

  pub fn as_integer(&self) -> Option<u64> {
    match self {
      Literal::Integer(u) => Some(*u),
      _ => None
    }
  }
  pub fn as_float(&self) -> Option<f64> {
    match self {
      Literal::Float(u) => Some(*u),
      _ => None
    }
  }
  pub fn as_char(&self) -> Option<u8> {
    match self {
      Literal::Char(u) => Some(*u),
      _ => None
    }
  }
  pub fn as_string(&self) -> Option<String> {
    match self {
      Literal::String(u) => Some(u.clone()),
      _ => None
    }
  }
}
