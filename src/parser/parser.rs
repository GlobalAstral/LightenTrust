
use std::collections::HashMap;

use crate::{constants::CONFIGS, parser::{expressions::{ExprKind, Expression}, nodes::{Fnc, Node}, types::{MemoryKind, Type, Variable}, utils::Processor}, tokens::token::{Token, TokenKind}};

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
      
      if block.len() <= 2 && block.len() > 0 {
        let size = block[0].as_literal().and_then(|l| l.as_integer())
          .unwrap_or_else(|| self.base.error("Expected Integer Literal"));
        let kind = if let Some(t) = block.get(1) {
          if t.kind == TokenKind::Dollar {
            MemoryKind::Float
          } else {
            MemoryKind::Integer
          }
        } else {
          MemoryKind::Integer
        };
        Some(Type::Memory { size, kind })
      
      } else if block.len() > 2 {
        let this: *mut Parser = self;
        
        let (size, r#type) = self.base.switch(block, |base| {
          let size = unsafe { (*this).parse_expr() };
          
          base.require(Token { kind: TokenKind::Semicolon, ..Default::default() });
          
          let r#type = unsafe { (*this).parse_type() }
            .unwrap_or_else(|| base.error("Expected Type"));
          
          (Box::new(size), r#type)
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

  fn parse_expr(&mut self) -> Expression {
    let expr = if self.base.tryconsume(Token { kind: TokenKind::SizeOf, ..Default::default() }) {
      Expression { kind: ExprKind::SizeOf(Box::new(self.parse_expr())), return_type: Type::Memory { size: CONFIGS.read().unwrap().ptr_size, kind: MemoryKind::Integer } }
    } else if self.base.tryconsume(Token { kind: TokenKind::Ampersand, ..Default::default() }) {
      let expr = self.parse_expr();
      Expression { return_type: Type::Pointer { r#type: Box::new(expr.return_type.clone()) }, kind: ExprKind::Reference(Box::new(expr)) }
    
    } else if matches!(self.base.peek().kind, TokenKind::Symbols(s) if s == "*") {
      self.base.consume();
      let expr = self.parse_expr();
    
      if let Type::Pointer { r#type } = &expr.return_type {
        return Expression { return_type: *r#type.clone(), kind: ExprKind::Dereference(Box::new(expr)) }
      }
      self.base.error("Cannot dereference a non pointer type");
    
    } else if matches!(self.base.peek().kind, TokenKind::ParenthesisBlock(_)) {
      let block = self.base.consume().as_paren_block().unwrap();
      let this: *mut Parser = self;
      self.base.switch(block, |_| unsafe { (*this).parse_expr() })
    
    } else if matches!(self.base.peek().kind, TokenKind::Literal(_)) {
      let lit = self.base.consume().as_literal().unwrap();
      println!("{:?}", lit);
      Expression { return_type: lit.get_type(), kind: ExprKind::Literal(lit)}
    
    } else if matches!(self.base.peek().kind, TokenKind::Identifier(_)) {
      let name = self.base.consume().as_identifier().unwrap().to_string();

      if matches!(self.base.peek().kind, TokenKind::ParenthesisBlock(_)) {
        let block = self.base.consume().as_paren_block().unwrap();
        let this: *mut Parser = self;
        
        let args =  self.base.switch(block, |base| {
          let mut args: Vec<Expression> = Vec::new();
          let mut count: usize = 0;
          while base.has_peek() {
            if count > 0 {
              base.require(Token { kind: TokenKind::Comma, ..Default::default() });
            }
            args.push( unsafe { (*this).parse_expr() } );
            count += 1;
          }
          args
        });

        let found_funcs: Vec<Fnc> = self.functions.iter()
        .filter(|f| {
          f.name == name &&
          f.arguments.len() == args.len() &&
          f.arguments.iter().zip(&args).all(|(x, y)| x.r#type == y.return_type)
        })
        .cloned()
        .collect();
        
        if found_funcs.is_empty() {
          self.base.error(&format!("Function {} does not exist with such arguments", name))
        }
        
        let function = if found_funcs.len() > 1 {
          self.base.require(Token { kind: TokenKind::Dollar, ..Default::default() });
          
          if let Some(t) = self.parse_type() {
            found_funcs.iter().find(|f| *f.return_type == t)
              .unwrap_or_else(|| self.base.error(&format!("Function {} does not exist with such arguments and specified type", name)))
          
          } else {
            self.base.error("Expected Type specifier");
          }
        
        } else {
          &found_funcs[0]
        };
        Expression { kind: ExprKind::FncCall { id: function.id, args }, return_type: *function.return_type.clone() }
      
      } else if self.functions.iter().find(|f| f.name == name).is_some() {
        let funcs: Vec<Fnc> = self.functions.iter().filter(|f| f.name == name).cloned().collect();
        
        if funcs.len() > 1 {
          
          if matches!(self.base.peek().kind, TokenKind::AngleBlock(_)) {
            let block = self.base.consume().as_angle_block().unwrap();
            let this: *mut Parser = self;
            let args = self.base.switch(block, |base| {
              let mut args: Vec<Type> = Vec::new();
              let mut count: usize = 0;
            
              while base.has_peek() {
            
                if count > 0 {
                  base.require(Token { kind: TokenKind::Comma, ..Default::default() });
                }
            
                args.push(unsafe { (*this).parse_type() }.unwrap_or_else(|| base.error("Expected Type")));
                count += 1;
              };
              args
            });
            
            let funcs: Vec<&Fnc> = funcs.iter().filter(|f| {
              f.arguments.len() == args.len() &&
              f.arguments.iter().zip(&args).all(|(x, y)| x.r#type == *y)
            }).collect();
            
            if funcs.is_empty() {
              self.base.error(&format!("Function {} with such arguments does not exist", name))
            }
            
            if funcs.len() == 1 {
              Expression { kind: ExprKind::FncPtrRef(funcs[0].id), return_type: *funcs[0].return_type.clone() }
            
            } else {
              self.base.require(Token { kind: TokenKind::Dollar, ..Default::default() });
              let t = self.parse_type().unwrap_or_else(|| self.base.error("Expected Type"));
            
              if let Some(f) = funcs.iter().find(|f| *f.return_type == t) {
                Expression { kind: ExprKind::FncPtrRef(f.id), return_type: *f.return_type.clone() }
            
              } else {
                self.base.error(&format!("Function {} with such arguments and return type does not exist", name))
              }
            }
          
          } else {
            self.base.error(&format!("Expected Type specifier for {}", name))
          }
        
        } else {
          Expression { kind: ExprKind::FncPtrRef(funcs[0].id), return_type: *funcs[0].return_type.clone() }
        }
      
      } else {
        let var = 
          if let Some(var) = self.globals.iter().find(|v| v.name == name) {
            var
      
          } else {
            self.locals.iter().find(|v| v.name == name)
              .unwrap_or_else(|| self.base.error(&format!("Variable {} does not exist", name)))
          };
        Expression { kind: ExprKind::Variable(var.id), return_type: var.r#type.clone() }
      }
    
    } else {
      self.base.error("Expected Expression");
    };

    if matches!(self.base.peek().kind, TokenKind::SquareBlock(_)) {
      if !matches!(expr.return_type, Type::Pointer { .. } | Type::Array { .. }) {
        self.base.error("Cannot index a non array or pointer type")
      }
      
      let block = self.base.consume().as_square_block().unwrap();
      let this: *mut Parser = self;
      
      let i = self.base.switch(block, |_| {
        unsafe { (*this).parse_expr() }
      });
      
      let t = if let Type::Array { r#type , ..} = &expr.return_type {
        r#type
      } else if let Type::Pointer { r#type } = &expr.return_type {
        r#type
      } else {
        unreachable!()
      };
      
      Expression { kind: ExprKind::Index { base: Box::new(expr.clone()), index: Box::new(i.clone()) }, return_type: *t.clone() }
    } else if matches!(self.base.peek().kind, TokenKind::ParenthesisBlock(_)) {
      if !matches!(expr.return_type, Type::FunctionPointer { .. }) {
        self.base.error("Only function pointers are callable");
      };
      
      let block = self.base.consume().as_paren_block().unwrap();
      let this: *mut Parser = self;
      
      let arguments = self.base.switch(block, |base| {
        let mut temp: Vec<Expression> = Vec::new();
        let mut count: usize = 0;
      
        while base.has_peek() {
          if count > 0 {
            base.require(Token { kind: TokenKind::Comma, ..Default::default() });
          }
      
          temp.push(unsafe {(*this).parse_expr()});
          count += 1;
        }
        temp
      });
      
      let (ret_type, args) = if let Type::FunctionPointer { return_type, arguments } = &expr.return_type {
        (return_type, arguments)
      } else {
        unreachable!()
      };
      
      if args.iter().zip(&arguments).all(|(x, y)| *x == y.return_type) {
        self.base.error("Invalid Arguments for Function Pointer");
      }
      
      Expression { kind: ExprKind::FncPtrCall { expr: Box::new(expr.clone()), args: arguments }, return_type: *ret_type.clone() }
    } else {
      expr
    }
  }

  fn parse_one(&mut self) -> Node {
    if matches!(self.base.peek().kind, TokenKind::CurlyBlock(_)) {
      let block = self.base.consume().as_curly_block().unwrap();
      self.scope_depth += 1;
      let before = self.locals.len();
      let this: *mut Parser = self;
      let vec = if !block.is_empty() { self.base.switch(block, |_| unsafe { (*this).parse() } ) } else { Vec::new() };
      self.scope_depth -= 1;
      self.locals.drain(before..);
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
      Node::Expr(self.parse_expr())
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
