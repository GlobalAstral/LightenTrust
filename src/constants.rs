use std::sync::RwLock;

use lazy_static::lazy_static;

pub static EXTENSION: &str = "lt";

#[derive(Debug, Clone, Default)]
pub struct Sizes {
  pub pointer: u64,
  pub intl_size: u64,
  pub floatl_size: u64,
  pub charl_size: u64,
}

#[derive(Debug, Clone, Default)]
pub struct SectionNames {
  pub data: String,
  pub bss: String,
  pub read_only: String,
  pub text: String
}

#[derive(Debug, Clone, Default)]
pub struct Configs {
  pub sizes: Sizes,
  pub sections: SectionNames,
  pub entry: String
}

lazy_static! {
  pub static ref CONFIGS: RwLock<Configs> = RwLock::new(Configs::default());
}

pub fn get_configs() -> Configs {
  CONFIGS.read().expect("Cannot read configs").clone()
}

pub static DEFAULT_CONFIG: &str = include_str!("./default_config.toml");
