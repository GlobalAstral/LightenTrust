use std::sync::RwLock;

use lazy_static::lazy_static;

pub static EXTENSION: &str = "lt";

#[derive(Debug, Clone)]
pub struct Configs {
  pub ptr_size: u64,
  pub intl_size: u64,
  pub floatl_size: u64,
  pub charl_size: u64,
  pub ro_sec_name: String,
  pub entry: String,
}

impl Default for Configs  {
  fn default() -> Self {
    Configs {
      ptr_size: 8,
      intl_size: 4,
      floatl_size: 4,
      charl_size: 1,
      ro_sec_name: ".rodata".into(),
      entry: "main".into()
    }
  }
}

lazy_static! {
  pub static ref CONFIGS: RwLock<Configs> = RwLock::new(Configs::default());
}

pub fn get_configs() -> Configs {
  CONFIGS.read().expect("Cannot read configs").clone()
}

pub static DEFAULT_CONFIG: &str = "ptr_size = 8\nfloat_lit = 4\nint_lit = 4\nchar_lit = 1\nro_sec_name = '.rodata'\nentry = 'main'";
