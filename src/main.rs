use std::{env, error::Error, fs::{self, OpenOptions}, io::Write, path::PathBuf};

use toml_edit::Document;

use crate::{constants::{CONFIGS, Configs, DEFAULT_CONFIG, EXTENSION, RegisterVariants, Registers, SectionNames, Sizes}, generator::generator::Generator, parser::parser::Parser, tokens::{preprocessor::Preprocessor, tokenizer::Tokenizer}};

mod constants;
mod tokens;
mod parser;
mod generator;

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
      sizes: {
        let szs = doc.get("sizes").and_then(|t| t.as_table()).expect("Cannot get table 'sizes'");
        Sizes {
          pointer: szs.get("pointer").and_then(|s| s.as_integer()).unwrap_or(8) as u64,
          intl_size: szs.get("int_lit").and_then(|s| s.as_integer()).unwrap_or(4) as u64, 
          floatl_size: szs.get("float_lit").and_then(|s| s.as_integer()).unwrap_or(4) as u64,
          charl_size: szs.get("char_lit").and_then(|s| s.as_integer()).unwrap_or(1) as u64, 
        }
      },
      sections: {
        let secs = doc.get("sections").and_then(|t| t.as_table()).expect("Cannot get table 'sections'");
        SectionNames {
          read_only: secs.get("read_only").and_then(|s| s.as_str()).unwrap_or(".rdata").into(),
          text: secs.get("text").and_then(|s| s.as_str()).unwrap_or(".text").into(),
          data: secs.get("data").and_then(|s| s.as_str()).unwrap_or(".data").into(),
          bss: secs.get("bss").and_then(|s| s.as_str()).unwrap_or(".bss").into(),
        }
      },
      entry: doc.get("entry").and_then(|s| s.as_str()).unwrap_or("main").into(),
      registers: {
        let regs = doc.get("registers").and_then(|t| t.as_table()).expect("Cannot get table 'registers'");
        let basic_regs = regs.get("basic").and_then(|l| l.as_array()).expect("Expected 'basic' array");
        let basic: Vec<RegisterVariants> = basic_regs.iter().map(|variant| {
            let array = variant.as_array().expect("Register variants must be an array");
            array.iter().map(|reg| reg.as_str().expect("Register variant must be string").to_string())
            .collect::<RegisterVariants>()
          }
        ).collect();
        let base_pointer: RegisterVariants = regs.get("base_pointer").and_then(|t| t.as_array()).expect("Expected 'base_pointer' array")
          .iter().map(|e| {
            e.as_str().expect("Register variant must be string").to_string()
          }).collect();
        
        let stack_pointer: RegisterVariants = regs.get("stack_pointer").and_then(|t| t.as_array()).expect("Expected 'stack_pointer' array")
          .iter().map(|e| {
            e.as_str().expect("Register variant must be string").to_string()
          }).collect();

        let return_register: RegisterVariants = regs.get("return_register").and_then(|t| t.as_array()).expect("Expected 'return_register' array")
        .iter().map(|e| {
          e.as_str().expect("Register variant must be string").to_string()
        }).collect();

        Registers {
          basic,
          base_pointer,
          stack_pointer,
          return_register
        }
      },
      biggest_size: {
        let regs = doc.get("registers").and_then(|t| t.as_table()).expect("Cannot get table 'registers'");
        regs.get("biggest_size").and_then(|t| t.as_integer()).unwrap_or(8) as usize
      }
    };
  }

  let content = fs::read_to_string(&input_file)?;
  let mut tokenizer: Tokenizer = Tokenizer::new(&content, input_file.clone());
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

  let mut generator: Generator = Generator::new(nodes, parser.globals); 
  println!("\nCOMPILED");

  let ret = generator.compile();
  println!("{}", ret);

  {
    let mut asm_file = OpenOptions::new()
      .create(true)
      .truncate(true)
      .write(true)
      .open(input_file.with_extension("asm"))?;
    write!(asm_file, "{}", ret)?;
  }
  Ok(())
}
