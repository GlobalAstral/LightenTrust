use std::sync::RwLock;

use lazy_static::lazy_static;

pub static EXTENSION: &str = "lt";

#[derive(Debug, Clone, Copy)]
pub struct Configs {
  pub ptr_size: u64,
  pub intl_size: u64,
  pub floatl_size: u64,
  pub charl_size: u64,
}

impl Default for Configs  {
  fn default() -> Self {
    Configs {
      ptr_size: 8,
      intl_size: 4,
      floatl_size: 4,
      charl_size: 1
    }
  }
}

lazy_static! {
  pub static ref CONFIGS: RwLock<Configs> = RwLock::new(Configs::default());
}

pub fn get_configs() -> Configs {
  CONFIGS.read().expect("Cannot read configs").clone()
}

pub static DEFAULT_CONFIG: &str =
  "
  ptr_size = 8\n
  float_lit = 4\n
  int_lit = 4\n
  char_lit = 1\n
  ";
