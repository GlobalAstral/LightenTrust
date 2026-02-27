use std::{collections::HashMap, path::PathBuf};

use crate::{constants::get_configs, parser::{expressions::{ExprKind, Expression}, literals::Literal, nodes::{Fnc, Node}, types::{Type, Variable}, utils::Processor}};

static mut LABEL_ID: u64 = 0;

#[derive(Debug, Default)]
pub struct Sections {
  pub text: String,
  pub data: String,
  pub bss: String,
  pub read_only: String,
}

pub enum MemoryLocation {
  Stack(isize),
  Register(String),
  Data(String)
}

impl MemoryLocation {
  pub fn get(self) -> String {
    match self {
      Self::Data(s) => format!("[{}]", s),
      Self::Register(reg) => reg,
      Self::Stack(ofs) => format!("[{}{}{}]", get_configs().registers.stack_pointer[0], if ofs > 0 {'+'} else {'-'}, ofs.abs())
    }
  }
}

pub struct Generator {
  pub base: Processor<Node>,
  pub sections: Sections,
  pub indent_depth: usize,
  pub used_registers: Vec<usize>,
  pub functions: Vec<Fnc>,
  pub globals: Vec<Variable>,
  pub vars: HashMap<u64, Option<Expression>>
}

impl Generator {
  pub fn new(i: Vec<Node>, globals: Vec<Variable>) -> Self {
    Self {
      base: Processor::new(i, Box::new(|_, _| false) , Box::new(|_| 0), Box::new(|_| PathBuf::new())), 
      sections: Sections::default(),
      indent_depth: 0,
      used_registers: Vec::new(),
      functions: Vec::new(),
      globals: globals,
      vars: HashMap::new()
    }
  }

  pub fn generate_label(&self) -> String {
    let temp = unsafe { LABEL_ID };
    unsafe {
      LABEL_ID += 1;
    };
    format!("L_{}", temp)
  }

  fn compile_expr(&mut self, expr: &Expression) -> MemoryLocation {
    match expr.kind {
      _ => unimplemented!()
    }
  }

  fn evaluate(&self, expr: &Expression) -> Literal {
    match &expr.kind {
      ExprKind::Literal(lit) => lit.clone(),
      ExprKind::Cast { base, ..} => self.evaluate(&base),
      ExprKind::Variable(var) => {
        if let Some(ex) = self.vars.get(&var).unwrap() {
          self.evaluate(ex)
        } else {
          self.base.error(&format!("Variable of id {} has no value", var));
        }
      },
      ExprKind::SizeOf(t) => Literal::Integer(t.return_type.get_size() as u64),
      _ => self.base.error(&format!("Expression {} is not evaluable at compiletime", expr))
    }
  }

  fn compile_one(&mut self, node: &Node) {
    match node {
      Node::Scope(s) | Node::Packet(s) => {
        s.iter().for_each(|node| {
          self.compile_one(node);
        });
      },
      Node::FncDecl(fnc) => {
        self.functions.push(fnc.clone());
        if let Some(body) = &fnc.body {
          self.create_function(&fnc.name, |this| {
            this.compile_one(&body);
          });
        }
      },
      Node::VariableDecl { var, expr } => {
        if var.global {
          if let Some(expr) = expr {
            if !expr.is_evaluable(&self.globals) {
              self.base.error(&format!("Expression {} is not constant", expr));
            }
            if var.mutable {
              match self.evaluate(expr) {
                Literal::Char(c) => self.init_alloc(&var.name, var.r#type.get_size(), &format!("{}", c)),
                Literal::Integer(i) => self.init_alloc(&var.name, var.r#type.get_size(), &format!("{}", i)),
                Literal::Float(f) => self.init_alloc(&var.name, var.r#type.get_size(), &format!("{}", f.to_bits())),
                Literal::String(s) => self.alloc_str(&var.name, &s),
              };
            } else {
              match self.evaluate(expr) {
                Literal::Char(c) => self.const_alloc(&var.name, var.r#type.get_size(), &format!("{}", c)),
                Literal::Integer(i) => self.const_alloc(&var.name, var.r#type.get_size(), &format!("{}", i)),
                Literal::Float(f) => self.const_alloc(&var.name, var.r#type.get_size(), &format!("{}", f.to_bits())),
                Literal::String(s) => self.alloc_str_const(&var.name, &s),
              };
            }
            self.vars.insert(var.id, Some(expr.clone()));
          } else {
            self.uninit_alloc(&var.name, var.r#type.get_size());
            self.vars.insert(var.id, None);
          }
        } else {
          unimplemented!("Not global variable declaration is not implemented");
        }
      }
      _ => unimplemented!()
    }
  }

  pub fn compile(&mut self) -> String {
    while self.base.has_peek() {
      let node = self.base.consume();
      self.compile_one(&node);
    }
    self.compose()
  }

  fn compose(&self) -> String {
    let configs = &get_configs();
    format!("global main\nsection {}\n{}\n\nsection {}\n{}\n\nsection {}\n{}\n\nsection {}\n{}\n",
      configs.sections.text,
      self.sections.text, 
      configs.sections.data,
      self.sections.data, 
      configs.sections.read_only,
      self.sections.read_only, 
      configs.sections.bss,
      self.sections.bss
    )
  }
}
