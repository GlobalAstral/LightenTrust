use std::fmt::Display;

use crate::{constants::get_configs, parser::expressions::Expression};

#[derive(Debug, PartialEq, PartialOrd, Clone)]
pub struct Variable {
  pub r#type: Type,
  pub name: String,
  pub id: u64,
  pub mutable: bool,
  pub global: bool,
}

impl Display for Variable  {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    write!(f, "{} {} {}<{}({})>", self.r#type, if self.mutable {"mut"} else {""}, self.name, self.id, if self.global {"global"} else {"local"})
  }
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone, Hash)]
pub enum MemoryKind {
  Unsigned,
  Integer,
  Float
}

impl Display for MemoryKind {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    match self {
      Self::Unsigned => write!(f, "ui"),
      Self::Integer => write!(f, "i"),
      Self::Float => write!(f, "f"),
    }
  }
}

#[derive(Debug, PartialEq, PartialOrd, Clone)]
pub enum Type {
  Alias {
    name: String,
    is: Box<Type>
  },
  Struct {
    fields: Vec<Variable>
  },
  Union {
    fields: Vec<Variable>
  },
  Array {
    size: Box<Expression>,
    r#type: Box<Type>
  },
  Pointer {
    r#type: Box<Type>
  },
  Memory {
    size: u64,
    kind: MemoryKind
  },
  FunctionPointer {
    return_type: Box<Type>,
    arguments: Vec<Type>
  }
}

impl Display for Type {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    match self {
      Self::Alias { name, is } => write!(f, "{}(actually {})", name, is),
      Self::Array { size, r#type } => write!(f, "[{}]({})", size, r#type),
      Self::Memory { size, kind } => write!(f, "[{}:{}]", size, kind),
      Self::Pointer { r#type } => write!(f, "&{}", r#type),
      Self::Struct { fields } => write!(f, "struct {{ {} }}", fields.iter().map(|v| format!("{}", v)).collect::<Vec<String>>().join("; ")),
      Self::Union { fields } => write!(f, "union {{ {} }}", fields.iter().map(|v| format!("{}", v)).collect::<Vec<String>>().join("; ")),
      Self::FunctionPointer { return_type, arguments } => write!(f, "fnc({}) {}", arguments.iter().map(|a| format!("{}", a)).collect::<Vec<String>>().join(", "), return_type)
    }
  }
}

impl Type {
  pub fn root(&self) -> &Type {
    match self {
      Self::Alias { is, .. } => is.root(),
      _ => self
    }
  }

  pub fn get_size(&self) -> usize {
    match self.root() {
      Self::Memory { size, .. } => *size as usize,
      Self::Array { .. } => get_configs().sizes.pointer as usize,
      Self::FunctionPointer { .. } => get_configs().sizes.pointer as usize,
      Self::Pointer { .. } => get_configs().sizes.pointer as usize,
      Self::Struct { fields } => fields.iter().fold(0, |a, b| a + b.r#type.get_size()),
      Self::Union { fields } => fields.iter().map(|f| f.r#type.get_size()).max().unwrap(),
      _ => {
        unreachable!()
      }
    }
  }

  pub fn get_align(&self) -> isize {
    match self.root() {
      Self::Memory { size, .. } => *size as isize,
      Self::Array { r#type, ..} => r#type.get_align() ,
      Self::FunctionPointer { .. } => get_configs().sizes.pointer as isize,
      Self::Pointer { .. } => get_configs().sizes.pointer as isize,
      Self::Struct { fields } | Self::Union { fields } => fields.iter().map(|f| f.r#type.get_align()).max().unwrap(),
      _ => {
        unreachable!()
      }
    }
  }
  
  pub fn compatible_with(&self, other: &Type) -> bool {
    if matches!(self, Self::Array { .. }) && matches!(other, Self::Pointer { .. }) {
      let Type::Array { r#type: t, .. } = self else { unreachable!() };
      let Type::Pointer { r#type: t2} = other else { unreachable!() };
      return t.compatible_with(&t2)
    }

    if matches!(self, Self::Struct { .. }) && matches!(other, Self::Struct { .. }) {
      let Type::Struct { fields: f1 } = self else { unreachable!() };
      let Type::Struct { fields: f2 } = other else { unreachable!() };
      return f2.len() == f2.len() && f1.iter().zip(f2).all(|(a, b)| a.r#type.compatible_with(&b.r#type))
    }
    
    if matches!(self, Self::Union { .. }) && matches!(other, Self::Union { .. }) {
      let Type::Union { fields: f1 } = self else { unreachable!() };
      let Type::Union { fields: f2 } = other else { unreachable!() };
      return f1.iter().zip(f2).all(|(a, b)| a.r#type.compatible_with(&b.r#type))
    }

    if matches!((self, other), (Self::Pointer { .. }, Self::Pointer { .. })) {
      return true;
    }

    let Type::Memory { size: s1, kind: k1 } = self.root() else {
      return false;
    };
    let Type::Memory { size: s2, kind: k2 } = other.root() else {
      return false;
    };

    let kinds_compatible = 
      if *k1 == MemoryKind::Integer && *k2 == MemoryKind::Unsigned {
        true
      } else if *k2 == MemoryKind::Integer && *k1 == MemoryKind::Unsigned {
        true
      } else {
        k1 == k2
      };

    kinds_compatible && s1 <= s2
  }
}
