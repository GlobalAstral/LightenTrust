use std::{collections::HashMap, fs, path::PathBuf};

use crate::{constants::get_configs, parser::utils::Processor, tokens::{token::{Token, TokenKind}, tokenizer::Tokenizer}};

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Macro {
  args: Vec<String>,
  content: Vec<Token>
}

pub struct Preprocessor {
  base: Processor<Token>,
  definitions: HashMap<String, Vec<Token>>,
  macros: HashMap<String, Macro>
}

impl Preprocessor {
  pub fn new(i: Vec<Token>) -> Self {
    Self {
      base: Processor::new(i, Box::new(|a, b| a.kind == b.kind), Box::new(|a| a.line), Box::new(|a| a.file.clone())),
      definitions: HashMap::new(),
      macros: HashMap::new()
    }
  }

  pub fn from(i: Vec<Token>, other: &Preprocessor) -> Self {
    Self {
      base: Processor::new(i, Box::new(|a, b| a.kind == b.kind), Box::new(|a| a.line), Box::new(|a| a.file.clone())),
      definitions: other.definitions.clone(), 
      macros: other.macros.clone()
    }
  }

  pub fn preprocess_id(&mut self, id: &str, line: usize, file: PathBuf, output: &mut Vec<Token>) {
    if self.definitions.contains_key(id) {
      let mut def = self.definitions.get(id).unwrap().clone();
      output.append(&mut def);
    } else if self.macros.contains_key(id) {
      let mcr = self.macros.get(id).unwrap().clone();
      let block = self.base.consume().as_paren_block()
        .unwrap_or_else(|| self.base.error("Expected ( ... ) block"));
      let args = self.base.switch(block, |base| {
        let mut arg: Vec<Token> = Vec::new();
        let mut args: Vec<Vec<Token>> = Vec::new();
        while base.has_peek() {
          if base.tryconsume(Token { kind: TokenKind::Comma, ..Default::default() }) {
            args.push(arg.clone());
            arg.clear();
          } else {
            arg.push(base.consume());
          }
        };
        if !arg.is_empty() {
          args.push(arg.clone());
        }
        args
      });
      if mcr.args.len() != args.len() {
        self.base.error("Invalid Arguments for Macro")
      }
      let restore = self.definitions.clone();
      for i in 0..mcr.args.len() {
        let id = &mcr.args[i];
        let content = args.iter().enumerate().nth(i).unwrap().1;
        self.definitions.insert(id.clone(), content.clone());
      }
      let this: *mut Preprocessor = self;
      let mut temp = self.base.switch(mcr.content, |_| {
        unsafe { (*this).preprocess() }
      });
      self.definitions = restore;
      output.append(&mut temp);
    } else {
      output.push(Token { kind: TokenKind::Identifier(id.to_string()), line, file });
    }
  }

  pub fn preprocess_dir(&mut self, output: &mut Vec<Token>) {
    if self.base.tryconsume(Token { kind: TokenKind::Include, ..Default::default() }) {
      let path = PathBuf::from(self.base.consume().as_literal().and_then(|l| l.as_string())
        .unwrap_or_else(|| self.base.error("Expected String Literal")));
      if !path.exists() {
        self.base.error(&format!("File {} does not exist", path.display()))
      }
      let content = fs::read_to_string(&path)
        .unwrap_or_else(|e| self.base.error(&format!("{}", e)));
      let mut tokenizer: Tokenizer = Tokenizer::new(&content, path);
      let tokens = tokenizer.tokenize().clone();
      let mut preprocessor: Preprocessor = Preprocessor::from(tokens, &self);
      let mut result = preprocessor.preprocess();
      output.append(&mut result);
    
    } else if self.base.tryconsume(Token { kind: TokenKind::Define, ..Default::default() }) {
      let identifier = self.base.consume().as_identifier()
        .unwrap_or_else(|| self.base.error("Expected Identifier")).to_string();
      if self.definitions.contains_key(&identifier) || self.macros.contains_key(&identifier) {
        self.base.error(&format!("Definition {} already exists", identifier));
      }
      let block = self.base.consume().as_curly_block()
        .unwrap_or_else(|| self.base.error("Expected { ... } block"));
      self.definitions.insert(identifier, block);
    
    } else if self.base.tryconsume(Token { kind: TokenKind::Macro, ..Default::default() }) {
      let identifier = self.base.consume().as_identifier()
        .unwrap_or_else(|| self.base.error("Expected Identifier")).to_string();
      if self.definitions.contains_key(&identifier) || self.macros.contains_key(&identifier) {
        self.base.error(&format!("Macro {} already exists", identifier));
      }
      let block = self.base.consume().as_paren_block()
        .unwrap_or_else(|| self.base.error("Expected Macro Arguments"));      
      let args = self.base.switch(block, |base| {
        let mut temp: Vec<String> = Vec::new();
        let mut count: usize = 0;
        while base.has_peek() {
          if count > 0 {
            base.require(Token { kind: TokenKind::Comma, ..Default::default() });
          }
          let id = base.consume().as_identifier()
            .unwrap_or_else(|| base.error("Expected Identifier")).to_string();
          temp.push(id);
          count += 1;
        };
        temp
      });

      let block = self.base.consume().as_curly_block()
        .unwrap_or_else(|| self.base.error("Expected { ... } block"));

      self.macros.insert(identifier, Macro { args, content: block });
    
    } else if self.base.tryconsume(Token { kind: TokenKind::Ifdef, ..Default::default() }) {
      let identifier = self.base.consume().as_identifier()
        .unwrap_or_else(|| self.base.error("Expected Identifier")).to_string();
      let block = self.base.consume().as_curly_block()
        .unwrap_or_else(|| self.base.error("Expected { ... } block"));
      if self.definitions.contains_key(&identifier) || self.macros.contains_key(&identifier) {
        let this: *mut Preprocessor = self;
        let mut temp = self.base.switch(block, |_| {
          unsafe { (*this).preprocess() }
        });
        output.append(&mut temp);
      }
    
    } else if self.base.tryconsume(Token { kind: TokenKind::Ifndef, ..Default::default() }) {
      let identifier = self.base.consume().as_identifier()
        .unwrap_or_else(|| self.base.error("Expected Identifier")).to_string();
      let block = self.base.consume().as_curly_block()
        .unwrap_or_else(|| self.base.error("Expected { ... } block"));
      if !self.definitions.contains_key(&identifier) && !self.macros.contains_key(&identifier) {
        let this: *mut Preprocessor = self;
        let mut temp = self.base.switch(block, |_| {
          unsafe { (*this).preprocess() }
        });
        output.append(&mut temp);
      }
    
    } else {
      self.base.error("Invalid preprocessor directive")
    }
  }

  pub fn preprocess_one(&mut self, line: usize, file: PathBuf, output: &mut Vec<Token>) {
    if !matches!(self.base.peek().kind, TokenKind::Hash | TokenKind::Identifier(_)) {
      output.push(self.base.consume());
      return;
    }

    if self.base.tryconsume(Token { kind: TokenKind::Hash, ..Default::default() }) {
      self.preprocess_dir(output);
    } else {
      let temp = self.base.consume().as_identifier().unwrap().to_string();
      self.preprocess_id(&temp, line, file, output);
    }
  }

  pub fn preprocess(&mut self) -> Vec<Token> {
    let mut output: Vec<Token> = Vec::new();

    while self.base.has_peek() {
      let line = self.base.peek().line;
      let file = self.base.peek().file;
      if self.base.tryconsume(Token { kind: TokenKind::GetConfig, ..Default::default() }) {
        let block = self.base.consume().as_paren_block()
          .unwrap_or_else(|| self.base.error("Expected ( ... ) block"));
        let size = self.base.switch(block, |base| {
          let name = base.consume().as_literal().and_then(|l| l.as_string())
          .unwrap_or_else(|| base.error("Expected String Literal"));
          base.require(Token { kind: TokenKind::Comma, ..Default::default() });
          let factor = base.consume().as_literal().and_then(|l| l.as_float())
            .unwrap_or_else(|| base.error("Expected Float Literal"));
          let size = match name.as_str() {
            "charl_size" => get_configs().charl_size,
            "floatl_size" => get_configs().floatl_size,
            "intl_size" => get_configs().intl_size,
            "ptr_size" => get_configs().ptr_size,
            _ => {
              base.error(&format!("Config {} does not exist", name));
            }
          } as f64 * factor;
          size as u64
        });
        
        output.push(Token { kind: TokenKind::Literal(size.to_string()), line, file });
        continue;
      }

      self.preprocess_one(line, file, &mut output);
    }

    output
  }
}
