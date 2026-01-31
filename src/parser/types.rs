use std::fmt::Display;

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone, Hash)]
pub struct Variable {
  pub r#type: Type,
  pub name: String,
  pub id: u64,
}

impl Display for Variable  {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    write!(f, "{} {}<{}>", self.r#type, self.name, self.id)
  }
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone, Hash)]
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
    size: u64,
    r#type: Box<Type>
  },
  Pointer {
    r#type: Box<Type>
  },
  Memory {
    size: u64
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
      Self::Memory { size } => write!(f, "[{}]", size),
      Self::Pointer { r#type } => write!(f, "&{}", r#type),
      Self::Struct { fields } => write!(f, "struct {{ {} }}", fields.iter().map(|v| format!("{}", v)).collect::<Vec<String>>().join("; ")),
      Self::Union { fields } => write!(f, "union {{ {} }}", fields.iter().map(|v| format!("{}", v)).collect::<Vec<String>>().join("; ")),
      Self::FunctionPointer { return_type, arguments } => write!(f, "fnc({}) {}", arguments.iter().map(|a| format!("{}", a)).collect::<Vec<String>>().join(", "), return_type)
    }
  }
}
