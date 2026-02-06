
use std::collections::HashMap;

use crate::{constants::CONFIGS, parser::{assembly::{AssemblyChunk, AssemblyParser}, expressions::{ExprKind, Expression, Operator}, nodes::{Fnc, Node}, types::{MemoryKind, Type, Variable}, utils::Processor}, tokens::token::{Token, TokenKind}};

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
  types: HashMap<String, Option<Type>>,
  locals: Vec<Variable>,
  globals: Vec<Variable>,
  scope_depth: usize,
  loop_depth: usize,
  functions: Vec<Fnc>,
  operators: Vec<Operator>,
  namespaces: Vec<String>,
  return_type: Option<Type>
}

impl Parser {
  pub fn new(i: Vec<Token>) -> Self {
    Self {
      base: Processor::new(i, Box::new(|a, b| a.kind == b.kind), Box::new(|s| s.line)), 
      types: HashMap::new(), 
      globals: Vec::new(), 
      locals: Vec::new(), 
      scope_depth: 0, 
      loop_depth: 0,
      functions: Vec::new(),
      operators: Vec::new(),
      namespaces: Vec::new(),
      return_type: None
    }
  }

  fn parse_var(&mut self, mutable: bool) -> Variable {
    if let Some(r#type) = self.parse_type() {
      if matches!(self.base.peek().kind, TokenKind::Identifier(_)) {
        let name = self.parse_identifier();
        return Variable { r#type, name, id: generate_id(), mutable: mutable }
      } else {
        self.base.error("Expected Identifier for Variable");
      }
    } else {
      self.base.error("Expected Type for Variable");
    }
  }

  fn parse_identifier(&mut self) -> String {
    let id = self.base.consume().as_identifier()
      .unwrap_or_else(|| self.base.error("Expected Identifier")).to_string();
    if self.namespaces.is_empty() {
      return id;
    }
    let mut temp = self.namespaces.join("::");
    temp.push_str("::");
    temp.push_str(&id);
    temp
  }

  fn require_identifier(&mut self) -> String {
    let mut temp = self.base.consume().as_identifier()
      .unwrap_or_else(|| self.base.error("Expected Identifier")).to_string();
    while self.base.tryconsume(Token { kind: TokenKind::Dot, ..Default::default() }) {
      temp.push_str("::");
      temp.push_str(
        self.base.consume().as_identifier()
        .unwrap_or_else(|| self.base.error("Expected Identifier"))
      );
    }

    temp
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
          } else if t.kind == TokenKind::Signed {
            MemoryKind::Integer
          } else {
            self.base.error("Unexpected MemoryKind specifier")
          }
        } else {
          MemoryKind::Unsigned
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
      let revert = self.base.peek;
      let id = self.require_identifier();
      if !self.types.contains_key(&id) {
        self.base.peek = revert;
        return None
      };
      
      let r#type = self.types.get(&id).cloned().unwrap();
      Some(Type::Alias { name: id, is: Box::new(r#type.unwrap()) })
    
    } else if self.base.tryconsume(Token { kind: TokenKind::Struct, ..Default::default() }) {
      if matches!(self.base.peek().kind, TokenKind::CurlyBlock(_)) {
        let block = self.base.consume().as_curly_block().unwrap();
        let mut vars: Vec<Variable> = Vec::new();
        let this: *mut Parser = self;
        
        self.base.switch(block, |base| {
          while base.has_peek() {
            vars.push(unsafe { (*this).parse_var(true) });
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
            vars.push(unsafe { (*this).parse_var(true) });
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
    let expr = if self.base.tryconsume(Token { kind: TokenKind::Symbols("*".into()), ..Default::default() }) {
      let expr = self.parse_expr();
      if let Type::Pointer { r#type } = &expr.return_type {
        let temp = *r#type.clone();
        return Expression { return_type: temp, kind: ExprKind::Dereference(Box::new(expr)) }
      }
      self.base.error("Cannot dereference a non pointer type");
    } else if matches!(self.base.peek().kind, TokenKind::Symbols(_)) {
      let symbols = self.base.consume().as_symbols().unwrap().to_string();
      let expr = self.parse_expr();
      let op = self.operators.iter().find(|op| {
        op.symbols == symbols &&
        op.right == None &&
        op.left.r#type.compatible_with(&expr.return_type)
      }).unwrap_or_else(|| self.base.error(&format!("Unary operator {} for {} does not exist", symbols, expr)));
      let temp = op.return_type.clone();
      Expression { kind: ExprKind::Unary { expr: Box::new(expr), operator:  op.clone()}, return_type: temp }
    } else if self.base.tryconsume(Token { kind: TokenKind::SizeOf, ..Default::default() }) {
      Expression { kind: ExprKind::SizeOf(Box::new(self.parse_expr())), return_type: Type::Memory { size: CONFIGS.read().unwrap().ptr_size, kind: MemoryKind::Unsigned } }
    } else if self.base.tryconsume(Token { kind: TokenKind::Ampersand, ..Default::default() }) {
      let expr = self.parse_expr();
      Expression { return_type: Type::Pointer { r#type: Box::new(expr.return_type.clone()) }, kind: ExprKind::Reference(Box::new(expr)) }
    
    } else if matches!(self.base.peek().kind, TokenKind::ParenthesisBlock(_)) {
      let block = self.base.consume().as_paren_block().unwrap();
      let this: *mut Parser = self;
      self.base.switch(block, |_| unsafe { (*this).parse_expr() })
    
    } else if matches!(self.base.peek().kind, TokenKind::Literal(_)) {
      let lit = self.base.consume().as_literal().unwrap();
      Expression { return_type: lit.get_type(), kind: ExprKind::Literal(lit)}
    
    } else if matches!(self.base.peek().kind, TokenKind::Identifier(_)) {
      let name = self.require_identifier();

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
          (f.variadic || f.arguments.len() == args.len()) &&
          f.arguments.iter().zip(&args[..f.arguments.len()]).all(|(x, y)| x.r#type.compatible_with(&y.return_type))
        })
        .cloned()
        .collect();
        
        if found_funcs.is_empty() {
          self.base.error(&format!("Function {} does not exist with such arguments", name))
        }
        
        let function = if found_funcs.len() > 1 {
          self.base.require(Token { kind: TokenKind::Dollar, ..Default::default() });
          
          if let Some(t) = self.parse_type() {
            found_funcs.iter().find(|f| f.return_type.compatible_with(&t))
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
              f.arguments.iter().zip(&args).all(|(x, y)| x.r#type.compatible_with(y))
            }).collect();
            
            if funcs.is_empty() {
              self.base.error(&format!("Function {} with such arguments does not exist", name))
            }
            
            if funcs.len() == 1 {
              Expression { kind: ExprKind::FncPtrRef(funcs[0].id), return_type: *funcs[0].return_type.clone() }
            
            } else {
              self.base.require(Token { kind: TokenKind::Dollar, ..Default::default() });
              let t = self.parse_type().unwrap_or_else(|| self.base.error("Expected Type"));
            
              if let Some(f) = funcs.iter().find(|f| f.return_type.compatible_with(&t)) {
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
      
      if args.iter().zip(&arguments).all(|(x, y)| x.compatible_with(&y.return_type)) {
        self.base.error("Invalid Arguments for Function Pointer");
      }
      
      Expression { kind: ExprKind::FncPtrCall { expr: Box::new(expr.clone()), args: arguments }, return_type: *ret_type.clone() }
    } else if self.base.tryconsume(Token { kind: TokenKind::Dot, ..Default::default() }) {
      if !matches!(expr.return_type, Type::Struct { .. } | Type::Union { .. }) {
        self.base.error("Cannot access a field of a non struct or union type");
      };
      
      let fields = if let Type::Struct { fields } = &expr.return_type {
        fields
      } else if let Type::Union { fields } = &expr.return_type {
        fields
      } else {
        unreachable!()
      }.clone();
      
      let name = self.base.consume().as_identifier()
        .unwrap_or_else(|| self.base.error("Expected Identifier")).to_string();

      let var = fields.iter().find(|v| v.name == name)
        .unwrap_or_else(|| self.base.error(&format!("No field {} exists for type", name)));

      Expression { return_type: var.r#type.clone(), kind: ExprKind::FieldAccess { base: Box::new(expr), field: var.clone() } }
    } else if self.base.tryconsume(Token { kind: TokenKind::To, ..Default::default() }) {
      let t = self.parse_type().unwrap_or_else(|| self.base.error("Expected Type"));
      if !expr.return_type.compatible_with(&t) {
        self.base.error(&format!("Type {} cannot be casted to a {} type", expr, t));
      }
      Expression { kind: ExprKind::Cast { base: Box::new(expr), into: t.clone() }, return_type: t }
    } else if self.base.tryconsume(Token { kind: TokenKind::Symbols("=".into()), ..Default::default() }) {
      if !matches!(expr.kind, ExprKind::Variable(_) | ExprKind::Dereference(_) | ExprKind::Index { .. } | ExprKind::FieldAccess { .. }) {
        self.base.error(&format!("{} is not a valid place for assignment", expr))
      };

      let value = self.parse_expr();
      if !value.return_type.compatible_with(&expr.return_type) {
        self.base.error(&format!("Value of type {} cannot be assigned to value of type {}", value.return_type, expr.return_type))
      };
      Expression { return_type: value.return_type.clone(), kind: ExprKind::Assignment { left: Box::new(expr), right: Box::new(value) } }
    } else if matches!(self.base.peek().kind, TokenKind::Symbols(_)) {
      let left = expr;
      let symbols = self.base.consume().as_symbols().unwrap().to_string();
      let right = self.parse_expr();
      let op = self.operators.iter().find(|op| {
        op.symbols == symbols &&
        op.right.as_ref().unwrap().r#type.compatible_with(&right.return_type) &&
        op.left.r#type.compatible_with(&left.return_type)
      }).unwrap_or_else(|| self.base.error(&format!("Binary operator {} for {} and {} does not exist", symbols, left, right)));
      if let ExprKind::Binary { left: l, right: r, operator: bin_op } = &right.kind {
        if op.precedence > bin_op.precedence {
          let bin: Expression = Expression {
            kind: ExprKind::Binary { left: Box::new(left), right: l.clone(), operator: op.clone() }, 
            return_type: op.return_type.clone()
          };
          let bin: Expression = Expression {
            kind: ExprKind::Binary { left: Box::new(bin), right: r.clone(), operator: bin_op.clone() },
            return_type: bin_op.return_type.clone()
          };
          bin
        } else {
          Expression {
            kind: ExprKind::Binary { left: Box::new(left), right: Box::new(Expression { kind: ExprKind::Binary { left: l.clone(), right: r.clone(), operator: bin_op.clone() }, return_type: bin_op.return_type.clone()}), operator: op.clone() }, 
            return_type: op.return_type.clone()
          }
        }
      } else {
        Expression { kind: ExprKind::Binary { left: Box::new(left), right: Box::new(right), operator: op.clone() }, return_type: op.return_type.clone() }
      }
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
      let name = self.parse_identifier();
      let (arguments, variadic)  = if matches!(self.base.peek().kind, TokenKind::ParenthesisBlock(_)) {
        let block = self.base.consume().as_paren_block().unwrap();
        let this: *mut Parser = self;
        
        self.base.switch(block, |base| {
          let mut count: usize = 0;
          let mut temp: Vec<Variable> = Vec::new();
          let mut variadic: bool = false;
          
          while base.has_peek() {
            if count > 0 {
              base.require(Token { kind: TokenKind::Comma, ..Default::default() });
            }
            if base.tryconsume(Token { kind: TokenKind::Ellipsis, ..Default::default() }) {
              variadic = true;
            } else {
              temp.push( unsafe { (*this).parse_var(false) } );
            }
            count += 1;
          }
          
          (temp, variadic)
        })
      
      } else {
        (Vec::new(), false)
      };
      
      let return_type = self.parse_type()
        .unwrap_or_else(|| self.base.error("Expected function return type"));

      let before = self.locals.len();
      arguments.iter().for_each(|arg| {
        self.locals.push(arg.clone());
      });

      let body = if self.base.tryconsume(Token { kind: TokenKind::Semicolon, ..Default::default() }) {
        None
      } else {
        self.return_type = Some(return_type.clone());
        let body = self.parse_one();
        if !matches!(body, Node::Scope(_)) {
          self.base.error("Scope for function body is mandatory")
        };
        self.return_type = None;
        Some(Box::new(body))
      };
      
      self.locals.drain(before..);
      
      let fnc = Fnc {name: name.clone(), return_type: Box::new(return_type.clone()), arguments: arguments.clone(), body: body, id: generate_id(), variadic, linkage: None};
      if self.functions.iter().find(|a| {
        a.name == name && 
        a.arguments.len() == arguments.len() && 
        a.arguments.iter().zip(&arguments).all(|(x, y)| x.r#type.compatible_with(&y.r#type)) &&
        a.return_type.compatible_with(&return_type) 
      }).is_some() {
        self.base.error(&format!("Function {} already exists", fnc))
      };
      
      self.functions.push(fnc.clone());
      Node::FncDecl(fnc)
    
    } else if self.base.tryconsume(Token { kind: TokenKind::Operator, ..Default::default() }) {
      let symbols = self.base.consume().as_symbols()
        .unwrap_or_else(|| self.base.error("Expected Operator symbol")).to_string();

      if !matches!(self.base.peek().kind, TokenKind::AngleBlock(_)) {
        self.base.error("Expected Operator Signature");
      }

      let block = self.base.consume().as_angle_block().unwrap();
      let this: *mut Parser = self;
      let (left, right, ret, prec) = self.base.switch(block, |base| {
        let left = unsafe {(*this).parse_var(false)};
        base.require(Token { kind: TokenKind::Comma, ..Default::default() });
        let right = if base.tryconsume(Token { kind: TokenKind::Dollar, ..Default::default() }) { None } 
          else { Some(unsafe {(*this).parse_var(false)}) };
        base.require(Token { kind: TokenKind::Comma, ..Default::default() });
        let return_type = unsafe {(*this).parse_type()}
          .unwrap_or_else(|| base.error("Expected Type"));
        base.require(Token { kind: TokenKind::Comma, ..Default::default() });
        let prec = base.consume().as_literal().and_then(|l| l.as_integer())
          .unwrap_or_else(|| base.error("Expected Integer Literal"));
        (left, right, return_type, prec)
      });
      let before = self.locals.len();
      self.locals.push(left.clone());
      if let Some(right) = &right {
        self.locals.push(right.clone());
      }
      let body = self.parse_one();
      if !matches!(body, Node::Scope(_)) {
        self.base.error("Expected Scope")
      }

      self.locals.drain(before..);

      let op: Operator = Operator {
        symbols, 
        left, 
        right, 
        return_type: ret, 
        precedence: prec, 
        body: Box::new(body)
      };
      let temp = self.operators.iter().find(|o| {
        o.symbols == op.symbols &&
        o.right.as_ref().map(|r| &r.r#type).unwrap().compatible_with(op.right.as_ref().map(|r| &r.r#type).unwrap()) &&
        o.left.r#type.compatible_with(&op.left.r#type)
      });
      if temp.is_some() {
        self.base.error("Operator already exists");
      };
      self.operators.push(op.clone());
      Node::OperatorDecl(op)
    } else if self.base.tryconsume(Token { kind: TokenKind::Typedef, ..Default::default() }) {
      let name = self.parse_identifier();
      if self.types.contains_key(&name) {
        self.base.error(&format!("Typedef {} already exists", &name));
      }
      self.types.insert(name.clone(), None);
      let t = self.parse_type()
        .unwrap_or_else(|| self.base.error("Expected Type"));
      self.base.require(Token { kind: TokenKind::Semicolon, ..Default::default() });
      self.types.insert(name, Some(t));
      Node::Ignored
    } else if let Some(r#type) = self.parse_type() {
      let mutable = self.base.tryconsume(Token { kind: TokenKind::Mut, ..Default::default() });
      let name = self.parse_identifier();
      if self.locals.iter().find(|v| v.name == name).is_some() || self.globals.iter().find(|v| v.name == name).is_some() {
        self.base.error(&format!("Variable {} already exists", name));
      }
      let var = Variable {id: generate_id(), mutable: mutable, name: name, r#type: r#type};
      if self.scope_depth > 0 {
        self.locals.push(var.clone());
      } else {
        self.globals.push(var.clone());
      };
      if self.base.tryconsume(Token { kind: TokenKind::Semicolon, ..Default::default() }) {
        Node::VariableDecl { var, expr: None }
      } else if matches!(self.base.peek().kind, TokenKind::Symbols(s) if s == "=") {
        self.base.consume();
        let expr = self.parse_expr();
        if !expr.return_type.compatible_with(&var.r#type) {
          self.base.error(&format!("Type {} cannot be assigned to type {}", &expr.return_type, &var.r#type));
        };
        self.base.require(Token { kind: TokenKind::Semicolon, ..Default::default() });
        Node::VariableDecl { var, expr: Some(expr) }
      } else {
        self.base.error("Expected EQUALS or SEMICOLON");
      }
    } else if self.base.tryconsume(Token { kind: TokenKind::Namespace, ..Default::default() }) {

      let id = self.base.consume().as_identifier()
        .unwrap_or_else(|| self.base.error("Expected Identifier")).to_string();

      self.namespaces.push(id.clone());

      let block = self.base.consume().as_curly_block()
        .unwrap_or_else(|| self.base.error(&format!("Expected namespace {} content", id)));

      let this: *mut Parser = self;
      let temp = self.base.switch(block, |_| {
        unsafe {(*this).parse()}
      });
      self.namespaces.pop();

      Node::Packet(temp)
    } else if self.base.tryconsume(Token { kind: TokenKind::Return, ..Default::default() }) {
      if self.return_type.is_none() {
        self.base.error("Cannot return outside of function");
      };
      let expr = self.parse_expr();
      if !expr.return_type.compatible_with(self.return_type.as_ref().unwrap()) {
        self.base.error(&format!("Type {} cannot be returned as a {} type", expr.return_type, self.return_type.as_ref().unwrap()));
      };
      self.base.require(Token { kind: TokenKind::Semicolon, ..Default::default() });
      Node::Return(expr)
    } else if self.base.tryconsume(Token { kind: TokenKind::Asm, ..Default::default() }) {
      let block = self.base.consume().as_curly_block()
        .unwrap_or_else(|| self.base.error("Expected { ... } block"));
      let asm_code: String = block.iter().map(
        |t| t.as_literal().and_then(|l| l.as_string())
          .unwrap_or_else(|| self.base.error("Expected String literal"))
      ).collect::<Vec<String>>().join("\n");

      let mut asm_parser: AssemblyParser = AssemblyParser::new(asm_code);
      let asm: Vec<AssemblyChunk> = asm_parser.parse(&self.locals, &self.globals)
        .unwrap_or_else(|e| self.base.error(&format!("{}", e))); 
      Node::Assembly(asm)
    } else if self.base.tryconsume(Token { kind: TokenKind::If, ..Default::default() }) {
      let expr = self.parse_expr();
      let body = self.parse_one();
      let else_body = if self.base.tryconsume(Token { kind: TokenKind::Else, ..Default::default() }) {
        Some(Box::new(self.parse_one()))
      } else {None};
      Node::If(expr, Box::new(body), else_body)
    } else if self.base.tryconsume(Token { kind: TokenKind::While, ..Default::default() }) {
      let expr = self.parse_expr();
      self.loop_depth += 1;
      let body = self.parse_one();
      self.loop_depth -= 1;
      Node::While(expr, Box::new(body))
    } else if self.base.tryconsume(Token { kind: TokenKind::Do, ..Default::default() }) {
      self.loop_depth += 1;
      let body = self.parse_one();
      self.loop_depth -= 1;
      self.base.require(Token { kind: TokenKind::While, ..Default::default() });
      let expr = self.parse_expr();
      self.base.require(Token { kind: TokenKind::Semicolon, ..Default::default() });
      Node::DoWhile(expr, Box::new(body))
    } else if self.base.tryconsume(Token { kind: TokenKind::For, ..Default::default() }) {
      let block = self.base.consume().as_paren_block()
        .unwrap_or_else(|| self.base.error("Expected for ( ... ) block"));
      
      let this: *mut Parser = self;
      let before = self.locals.len();
      
      let (var, init, cond, incr) = self.base.switch(block, |base| {
        let var = unsafe { (*this).parse_var(true) };
        base.require(Token { kind: TokenKind::Symbols(String::from("=")), ..Default::default() });
        let expr = unsafe { (*this).parse_expr() };
        self.locals.push(var.clone());
        base.require(Token { kind: TokenKind::Semicolon, ..Default::default() });
        let cond = unsafe { (*this).parse_expr() };
        base.require(Token { kind: TokenKind::Semicolon, ..Default::default() });
        let incr = unsafe { (*this).parse_one() };
        (var, expr, cond, incr)
      });
      self.loop_depth += 1;
      let body = self.parse_one();
      self.loop_depth -= 1;
      self.locals.drain(before..);
      Node::For { var, init, cond, incr: Box::new(incr), body: Box::new(body) }
    } else if self.base.tryconsume(Token { kind: TokenKind::Break, ..Default::default() }) {
      if self.loop_depth > 0 {
        return Node::Break;
      }
      self.base.error("Cannot break outside of a loop");
    } else if self.base.tryconsume(Token { kind: TokenKind::Continue, ..Default::default() }) {
      if self.loop_depth > 0 {
        return Node::Continue;
      }
      self.base.error("Cannot continue outside of a loop");
    } else {
      let temp = self.parse_expr();
      self.base.require(Token { kind: TokenKind::Semicolon, ..Default::default() });
      Node::Expr(temp)
    }
  }

  pub fn parse(&mut self) -> Vec<Node> {
    let mut ret: Vec<Node> = Vec::new();
    while self.base.has_peek() {
      let t = self.parse_one();
      if matches!(t, Node::Invalid) {
        self.base.error(&format!("Invalid Statement {}", t))
      }
      if !matches!(t, Node::Ignored) {
        ret.push(t);
      }
    }
    return ret;
  }
}
