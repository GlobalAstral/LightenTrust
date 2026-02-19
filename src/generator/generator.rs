use std::path::PathBuf;

use crate::{constants::CONFIGS, parser::{nodes::Node, utils::Processor}};

#[derive(Debug, Default)]
pub struct Sections {
  pub text: String,
  pub data: String,
  pub bss: String,
  pub read_only: String,
}

pub struct Generator {
  pub base: Processor<Node>,
  pub sections: Sections
}

impl Generator {
  pub fn new(i: Vec<Node>) -> Self {
    Self {
      base: Processor::new(i, Box::new(|a, b| std::mem::discriminant(a) == std::mem::discriminant(b)) , Box::new(|_| 0), Box::new(|_| PathBuf::new())), 
      sections: Sections::default()
    }
  }

  fn compose(&self) -> String {
    let configs = CONFIGS.read().unwrap();
    format!("global main\nsection .text\n{}\n\nsection .data\n{}\n\nsection {}\n{}\n\nsection .bss\n{}\n",
      self.sections.text, self.sections.data, configs.ro_sec_name, self.sections.read_only, self.sections.bss
    )
  }
}
