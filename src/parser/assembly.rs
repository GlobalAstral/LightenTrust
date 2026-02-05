use std::{error::Error, fmt::{Display, Formatter}, iter::Peekable};

use crate::parser::types::Variable;

#[derive(Debug, PartialEq, PartialOrd, Clone)]
pub enum AssemblyChunk {
  Original(String),
  Var(u64)
}

#[derive(Debug)]
pub enum AssemblyParseError {
  UnterminatedPlaceholder,
  UnknownVariable(String),
}

impl Display for AssemblyParseError {
  fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
    match self {
      AssemblyParseError::UnterminatedPlaceholder => write!(f, "placeholder was opened with '{{' but never closed with '}}'"),
      AssemblyParseError::UnknownVariable(name) => write!(f, "unknown variable in placeholder: '{}'", name)
    }
  }
}

impl Error for AssemblyParseError { }

pub struct AssemblyParser {
  code: Peekable<std::vec::IntoIter<char>>,
}

impl AssemblyParser {
  pub fn new(i: String) -> Self {
    let temp: Peekable<std::vec::IntoIter<char>> = i.chars().collect::<Vec<_>>().into_iter().peekable();
    Self { code: temp }
  }

  pub fn parse(&mut self, locals: &[Variable], globals: &[Variable]) -> Result<Vec<AssemblyChunk>, AssemblyParseError> {
    let mut original: String = String::new();
    let mut output: Vec<AssemblyChunk> = Vec::new();
    while let Some(ch) = self.code.next() {
      match ch {
        '{' => {
          if !original.is_empty() {
            output.push(AssemblyChunk::Original(original.clone()));
            original.clear();
          }
          let mut found = false;
          let mut buf: String = String::new();
          while let Some(ch) = self.code.next() {
            if ch == '}' {
              found = true;
              break;
            }
            buf.push(ch);
          }
          if !found {
            return Err(AssemblyParseError::UnterminatedPlaceholder);
          };
          let Some(var) = locals.iter().find(|v| v.name == buf)
            .or_else(|| globals.iter().find(|v| v.name == buf)) else {
              return Err(AssemblyParseError::UnknownVariable(buf));
            };
          output.push(AssemblyChunk::Var(var.id));
        },
        c => {
          original.push(c);
        }
      }
    };
    if !original.is_empty() {
      output.push(AssemblyChunk::Original(original));
    }
    Ok(output)
  }
}
