use std::{collections::HashMap, path::PathBuf};

use crate::{constants::get_configs, parser::{expressions::{ExprKind, Expression}, literals::Literal, nodes::{Fnc, Node}, types::{Type, Variable}, utils::Processor}};

static mut LABEL_ID: u64 = 0;

#[derive(Debug, Default, Clone)]
pub struct Sections {
  pub text: String,
  pub data: String,
  pub bss: String,
  pub read_only: String,
}

pub enum MemoryLocation {
  Stack(isize),
  Register(String),
  Data(String),
  Value(String)
}

pub struct StackFrame {
  pub next_ofs: isize,
  pub locals: HashMap<u64, isize>
}

impl StackFrame {
  pub fn new() -> Self {
    Self { next_ofs: 0, locals: HashMap::new() }
  }
}

impl MemoryLocation {
  pub fn get(self) -> String {
    match self {
      Self::Data(s) => format!("[{}]", s),
      Self::Register(reg) => reg,
      Self::Stack(ofs) => if ofs == 0 {
        format!("[{}]", get_configs().registers.base_pointer[0])
      } else {
        format!("[{}{}{}]", get_configs().registers.base_pointer[0], if ofs > 0 {'+'} else {'-'}, ofs.abs())
      },
      Self::Value(val) => val
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
  pub vars: HashMap<u64, Option<Expression>>,
  pub stack_frames: Vec<StackFrame>,
  pub selected_stack_frame: isize,
  pub free_cache: Vec<usize>,
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
      vars: HashMap::new(),
      stack_frames: Vec::new(),
      selected_stack_frame: -1,
      free_cache: Vec::new()
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
    match &expr.kind {
      ExprKind::Literal(lit) => match lit {
        Literal::Char(ch) => MemoryLocation::Value(ch.to_string()),
        Literal::Integer(i) => MemoryLocation::Value(i.to_string()),
        Literal::String(s) => {
          let lbl = self.generate_label();
          self.alloc_str_const(&lbl, &s);
          let reg = self.get_ret_reg(get_configs().sizes.pointer as usize);
          self.lea(&reg, &format!("[rel {}]", lbl));
          MemoryLocation::Register(reg)
        },
        Literal::Float(f) => {
          let lbl = self.generate_label();
          let fsize = get_configs().sizes.floatl_size as usize;
          self.const_alloc(&lbl, fsize, &f.to_string());
          let (simd, id) = self.get_unused_register(fsize, true);
          self.free_cache.push(id);
          self.movss(&simd, &format!("[{}]", lbl));
          MemoryLocation::Register(simd)
        }
      },

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
          let val = if let Some(expr) = expr {
            self.compile_expr(expr).get()
          } else {
            String::from("0")
          };
          self.alloc_var(var.id, -(var.r#type.get_size() as isize), var.r#type.get_align(), &val);
          self.free_cache();
        }
      },

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
    format!("global {}\nsection {}\n{}\n\nsection {}\n{}\n\nsection {}\n{}\n\nsection {}\n{}\n",
      configs.entry,
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
