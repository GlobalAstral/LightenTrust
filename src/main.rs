use std::{env, error::Error, fs, path::PathBuf};

use crate::{constants::EXTENSION, tokens::tokenizer::Tokenizer};

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

  let content = fs::read_to_string(input_file)?;
  let mut tokenizer: Tokenizer = Tokenizer::new(&content);
  let tokens = tokenizer.tokenize();
  tokens.iter().for_each(|t| {
    println!("{}", t)
  });

  Ok(())
}
