use std::path::PathBuf;

use crate::{constants::get_configs, parser::{expressions::{ExprKind, Expression}, nodes::Node, types::Type, utils::Processor}};

static mut LABEL_ID: u64 = 0;

#[derive(Debug, Default)]
pub struct Sections {
  pub text: String,
  pub data: String,
  pub bss: String,
  pub read_only: String,
}

pub enum MemoryLocation {
  Stack(isize),
  Register(String),
  Data(String)
}

impl MemoryLocation {
  pub fn get(self) -> String {
    match self {
      Self::Data(s) => format!("[{}]", s),
      Self::Register(reg) => reg,
      Self::Stack(ofs) => format!("[rsp{}{}]", if ofs > 0 {'+'} else {'-'}, ofs.abs())
    }
  }
}

pub struct Generator {
  pub base: Processor<Node>,
  pub sections: Sections,
  pub indent_depth: usize,
}

impl Generator {
  pub fn new(i: Vec<Node>) -> Self {
    Self {
      base: Processor::new(i, Box::new(|_, _| false) , Box::new(|_| 0), Box::new(|_| PathBuf::new())), 
      sections: Sections::default(),
      indent_depth: 0
    }
  }

  pub fn generate_label(&self) -> String {
    let temp = unsafe { LABEL_ID };
    unsafe {
      LABEL_ID += 1;
    };
    format!("L_{}", temp)
  }

  fn compile_expr(&mut self, expr: &Expression) -> MemoryLocation {
    match expr.kind {
      _ => unimplemented!()
    }
  }

  fn compile_one(&mut self, node: &Node) {
    match node {
      Node::Scope(s) | Node::Packet(s) => {
        s.iter().for_each(|node| {
          self.compile_one(node);
        });
      },

      _ => unimplemented!()
    }
  }

  pub fn compile(&mut self) -> String {
    while self.base.has_peek() {
      let node = self.base.consume();
      self.compile_one(&node);
    }
    self.compose()
  }

  fn compose(&self) -> String {
    let configs = &get_configs();
    format!("global main\nsection {}\n{}\n\nsection {}\n{}\n\nsection {}\n{}\n\nsection {}\n{}\n",
      configs.sections.text,
      self.sections.text, 
      configs.sections.data,
      self.sections.data, 
      configs.sections.read_only,
      self.sections.read_only, 
      configs.sections.bss,
      self.sections.bss
    )
  }
}
