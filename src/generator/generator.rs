use std::path::PathBuf;

use toml_edit::Document;

use crate::{constants::CONFIGS, parser::{nodes::Node, utils::Processor}};

#[derive(Debug, Default)]
struct Sections {
  text: String,
  data: String,
  bss: String,
  read_only: String,
}

pub struct Generator {
  base: Processor<Node>,
  sections: Sections
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
    format!("global {}\nsection .text\n{}\n\nsection .data\n{}\n\nsection {}\n{}\n\nsection .bss\n{}\n",
      configs.entry, self.sections.text, self.sections.data, configs.ro_sec_name, self.sections.read_only, self.sections.bss
    )
  }
}
