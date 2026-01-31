
use std::collections::HashMap;

use crate::{parser::{nodes::{Fnc, Node}, types::{Type, Variable}, utils::Processor}, tokens::token::{Token, TokenKind}};

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
  types: HashMap<String, Type>,
  locals: Vec<Variable>,
  globals: Vec<Variable>,
  scope_depth: usize,
  functions: Vec<Fnc>,
}

impl Parser {
  pub fn new(i: Vec<Token>) -> Self {
    Self {
      base: Processor::new(i, Box::new(|a, b| a.kind == b.kind), Box::new(|s| s.line)), 
      types: HashMap::new(), 
      globals: Vec::new(), 
      locals: Vec::new(), 
      scope_depth: 0, 
      functions: Vec::new()
    }
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
          unsafe { (*this).parse_type() }.unwrap_or_else(|| base.error("Expected Type"))
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
    } else if self.base.tryconsume(Token { kind: TokenKind::Fnc, ..Default::default() }) {
      if !matches!(self.base.peek().kind, TokenKind::ParenthesisBlock(_)) {
        self.base.error("Expected arguments of function pointer type");
      };
      let block = self.base.consume().as_paren_block().unwrap();
      let return_type = self.parse_type()
        .unwrap_or_else(|| self.base.error("Expected return type of function pointer type"));
      let mut types: Vec<Type> = Vec::new();
      if !block.is_empty() {
        let this: *mut Parser = self;
        self.base.switch(block, |base| {
          let mut count: usize = 0;
          while base.has_peek() {
            if count > 0 {
              base.require(Token { kind: TokenKind::Comma, ..Default::default() });
            };
            types.push(unsafe { (*this).parse_type() }.unwrap_or_else(|| base.error("Expected argument type")));
            count += 1;
          }
        });
      }
      Some(Type::FunctionPointer { return_type: Box::new(return_type), arguments: types })
    } else {
      None
    }
  }

  fn parse_one(&mut self) -> Node {
    if matches!(self.base.peek().kind, TokenKind::CurlyBlock(_)) {
      let block = self.base.consume().as_curly_block().unwrap();
      self.scope_depth += 1;
      let this: *mut Parser = self;
      let vec = if !block.is_empty() { self.base.switch(block, |_| unsafe { (*this).parse() } ) } else { Vec::new() };
      self.scope_depth -= 1;
      self.locals.clear();
      Node::Scope(vec)
    } else if self.base.tryconsume(Token { kind: TokenKind::Fnc, ..Default::default() }) {
      let name = self.base.consume().as_identifier()
        .unwrap_or_else(|| self.base.error("Expected Identifier for function name")).to_string();
      let arguments: Vec<Variable> = if matches!(self.base.peek().kind, TokenKind::ParenthesisBlock(_)) {
        let block = self.base.consume().as_paren_block().unwrap();
        let this: *mut Parser = self;
        self.base.switch(block, |base| {
          let mut count: usize = 0;
          let mut temp: Vec<Variable> = Vec::new();
          while base.has_peek() {
            if count > 0 {
              base.require(Token { kind: TokenKind::Comma, ..Default::default() });
            }
            temp.push( unsafe { (*this).parse_var() } );
            count += 1;
          }
          temp
        })
      } else {
        Vec::new()
      };
      let return_type = self.parse_type()
        .unwrap_or_else(|| self.base.error("Expected function return type"));
      arguments.iter().for_each(|arg| {
        self.locals.push(arg.clone());
      });
      let body = self.parse_one();
      if !matches!(body, Node::Scope(_)) {
        self.base.error("Scope for function body is mandatory")
      };
      let fnc = Fnc {name: name.clone(), return_type: Box::new(return_type.clone()), arguments: arguments.clone(), body: Box::new(body.clone()), id: generate_id()};
      if self.functions.iter().find(|a| {
        a.name == name && 
        a.arguments.len() == arguments.len() && 
        a.arguments.iter().zip(&arguments).all(|(x, y)| x.r#type == y.r#type) &&
        *a.return_type == return_type
      }).is_some() {
        self.base.error(&format!("Function {} already exists", fnc))
      };
      self.functions.push(fnc.clone());
      Node::FncDecl(fnc)
    } else {
      self.base.error("Invalid Statement");
    }
  }

  pub fn parse(&mut self) -> Vec<Node> {
    let mut ret: Vec<Node> = Vec::new();
    while self.base.has_peek() {
      ret.push(self.parse_one());
    }
    return ret;
  }
}
