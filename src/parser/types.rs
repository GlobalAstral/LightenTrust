
#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone, Hash)]
pub struct Variable {
  pub r#type: Type,
  pub name: String,
  pub id: u64,
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
  Enum {
    r#type: Box<Type>,
    entries: Vec<String>
  },
  Array {
    size: usize,
    r#type: Box<Type>
  },
  Pointer {
    r#type: Box<Type>
  },
  Memory {
    size: usize
  }
}
