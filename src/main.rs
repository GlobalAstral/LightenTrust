use std::{env, error::Error, fs::{self, OpenOptions}, io::Write, path::PathBuf};

use toml_edit::Document;

use crate::{constants::{CONFIGS, Configs, DEFAULT_CONFIG, EXTENSION}, parser::parser::Parser, tokens::{preprocessor::Preprocessor, tokenizer::Tokenizer}};

mod constants;
mod tokens;
mod parser;

fn main() -> Result<(), Box<dyn Error>> {
  let args: Vec<String> = env::args().collect();

  let input_file: Result<&String, Box<dyn Error>> = args.get(1).ok_or_else(|| "Invalid CLI arguments".into());
  let mut input_file = PathBuf::from(input_file?);
  input_file.set_extension(EXTENSION);
  let args: Vec<&str> = args.iter().skip(2).map(|s| s.as_str()).collect();

  #[allow(unused_variables)]
  let output_file = if let Some((index, _)) = args.iter().enumerate().find(|(_, arg)| **arg == "-o") {
    let temp: Result<&&str, Box<dyn Error>> = args.get(index + 1).ok_or_else(|| "Invalid CLI arguments".into());
    let mut temp = PathBuf::from(temp?);
    temp.set_extension("exe");
    temp
  } else {
    input_file.with_extension("exe")
  };

  let config_file: PathBuf = 
    if let Some((index, _)) = args.iter().enumerate().find(|(_, arg)| **arg == "-cfg") {
      let temp: Result<&&str, Box<dyn Error>> = args.get(index + 1).ok_or_else(|| "Invalid CLI arguments".into());
      let mut temp = PathBuf::from(temp?);
      temp.set_extension("toml");
      temp
    } else {
      PathBuf::from("./config.toml")
    };
  
  if !config_file.exists() {
    let mut f = OpenOptions::new()
      .create_new(true)
      .write(true)
      .open(&config_file)?;
    write!(f, "{}", DEFAULT_CONFIG)?;
  }

  {
    let doc: Document<String> = fs::read_to_string(config_file)?.parse()?;
    let mut writer = CONFIGS.write()?;
    *writer = Configs {
      ptr_size: doc.get("ptr_size").unwrap().as_integer().unwrap_or(8) as u64, 
      intl_size: doc.get("int_lit").unwrap().as_integer().unwrap_or(4) as u64, 
      floatl_size: doc.get("float_lit").unwrap().as_integer().unwrap_or(4) as u64,
      charl_size: doc.get("char_lit").unwrap().as_integer().unwrap_or(1) as u64, 
    };
  }

  let content = fs::read_to_string(input_file)?;
  let mut tokenizer: Tokenizer = Tokenizer::new(&content);
  let tokens = tokenizer.tokenize();
  println!("TOKENS");
  tokens.iter().for_each(|t| {
    println!("{}", t)
  });

  let mut preprocessor: Preprocessor = Preprocessor::new(tokens.clone());
  let tokens = preprocessor.preprocess();

  println!("PROCESSED TOKENS");
  tokens.iter().for_each(|t| {
    println!("{}", t)
  });

  let mut parser: Parser = Parser::new(tokens.clone());

  println!("\nPARSED");
  let nodes = parser.parse();
  nodes.iter().for_each(|n| {
    println!("{}", n);
  });

  Ok(())
}
