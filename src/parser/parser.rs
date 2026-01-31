
use std::collections::HashMap;

use crate::{parser::{types::{Type, Variable}, utils::Processor}, tokens::token::{Token, TokenKind}};

static mut CURRENT_ID: u64 = 0;

fn generate_id() -> u64 {
  unsafe {
    let temp = CURRENT_ID;
    CURRENT_ID += 1;
    return temp;
  }
}

pub struct Parser {
  base: Processor<Token>,
  types: HashMap<String, Type>
}

impl Parser {
  pub fn new(i: Vec<Token>) -> Self {
    Self { base: Processor::new(i, Box::new(|a, b| a.kind == b.kind), Box::new(|s| s.line)), types: HashMap::new() }
  }

  fn parse_var(&mut self) -> Variable {
    if let Some(r#type) = self.parse_type() {
      if matches!(self.base.peek().kind, TokenKind::Identifier(_)) {
        let name = self.base.consume().as_identifier().unwrap().to_string();
        return Variable { r#type, name, id: generate_id() }
      } else {
        self.base.error("Expected Identifier for Variable");
      }
    } else {
      self.base.error("Expected Type for Variable");
    }
  }

  fn parse_type(&mut self) -> Option<Type> {
    if self.base.tryconsume(Token { kind: TokenKind::Ampersand, ..Default::default() }) {
      Some(Type::Pointer { r#type: Box::new(self.parse_type().unwrap_or_else(|| self.base.error("Expected Type"))) })
    } else if matches!(self.base.peek().kind, TokenKind::SquareBlock(_)) {
      let block = self.base.consume().as_square_block().unwrap();
      if block.len() == 1 {
        let size = block[0].as_literal().and_then(|l| l.as_integer()).unwrap_or_else(|| self.base.error("Expected Integer Literal"));
        Some(Type::Memory { size })
      } else if block.len() > 2 {
        let size = block[0].as_literal().and_then(|l| l.as_integer()).unwrap_or_else(|| self.base.error("Expected Integer Literal"));
        if block[1].kind != TokenKind::Semicolon { self.base.error("Expected Semicolon for Array type") };
        let temp: Vec<Token> = block[2..].iter().map(|e| e.clone()).collect();
        let this: *mut Parser = self;
        let r#type = self.base.switch(temp, |base| {
          unsafe { (*this).parse_type().unwrap_or_else(|| base.error("Expected Type")) }
        });
        Some(Type::Array { size, r#type: Box::new(r#type) })
      } else {
        self.base.error("Only array or memory type can be described with []")
      }
    } else if matches!(self.base.peek().kind, TokenKind::Identifier(_)) {
      let id = self.base.consume().as_identifier().unwrap().to_string();
      if !self.types.contains_key(&id) {
        self.base.error(&format!("Type {} does not exist", id));
      };
      let r#type = self.types.get(&id).cloned().unwrap();
      Some(Type::Alias { name: id, is: Box::new(r#type) })
    } else if self.base.tryconsume(Token { kind: TokenKind::Struct, ..Default::default() }) {
      if matches!(self.base.peek().kind, TokenKind::CurlyBlock(_)) {
        let block = self.base.consume().as_curly_block().unwrap();
        let mut vars: Vec<Variable> = Vec::new();
        let this: *mut Parser = self;
        self.base.switch(block, |base| {
          while base.has_peek() {
            vars.push(unsafe { (*this).parse_var() });
            base.require(Token { kind: TokenKind::Semicolon, ..Default::default() });
          }
        });
        Some(Type::Struct { fields: vars })
      } else {
        self.base.error("Expected Fields for struct")
      }
    } else if self.base.tryconsume(Token { kind: TokenKind::Union, ..Default::default() }) {
      if matches!(self.base.peek().kind, TokenKind::CurlyBlock(_)) {
        let block = self.base.consume().as_curly_block().unwrap();
        let mut vars: Vec<Variable> = Vec::new();
        let this: *mut Parser = self;
        self.base.switch(block, |base| {
          while base.has_peek() {
            vars.push(unsafe { (*this).parse_var() });
            base.require(Token { kind: TokenKind::Semicolon, ..Default::default() });
          }
        });
        Some(Type::Union { fields: vars })
      } else {
        self.base.error("Expected Fields for struct")
      }
    } else {
      None
    }
  }

  pub fn parse(&mut self) {
    while self.base.has_peek() {

    }
  }
}
