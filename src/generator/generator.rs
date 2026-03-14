use std::{collections::HashMap, path::PathBuf};

use crate::{constants::get_configs, parser::{assembly::AssemblyChunk, expressions::{ExprKind, Expression}, literals::Literal, nodes::{Fnc, Node}, types::{Type, Variable}, utils::Processor}, scanner::scanner::StackFrame};

static mut LABEL_ID: u64 = 0;

#[derive(Debug, Default, Clone)]
pub struct Sections {
  pub text: String,
  pub data: String,
  pub bss: String,
  pub read_only: String,
}

#[derive(Debug, Clone)]
pub enum MemoryLocation {
  Stack(isize),
  Register(String),
  Data(String),
  Value(String)
}

impl MemoryLocation {
  pub fn get(&self) -> String {
    match self {
      Self::Data(s) => format!("[{}]", s),
      Self::Register(reg) => reg.clone(),
      Self::Stack(ofs) => if *ofs == 0 {
        format!("[{}]", get_configs().registers.base_pointer[0])
      } else {
        format!("[{}{}{}]", get_configs().registers.base_pointer[0], if *ofs > 0 {'+'} else {'-'}, ofs.abs())
      },
      Self::Value(val) => val.clone()
    }
  }
}

#[derive(Debug, Clone)]
pub struct VarContext {
  expr: Option<Expression>,
  location: MemoryLocation,
  r#type: Type
}

pub struct Generator {
  pub base: Processor<Node>,
  pub sections: Sections,
  pub indent_depth: usize,
  pub used_registers: Vec<usize>,
  pub functions: Vec<Fnc>,
  pub globals: Vec<Variable>,
  pub vars: HashMap<u64, VarContext>,
  pub stack_frames: Vec<StackFrame>,
  pub selected_stack_frame: isize,
  pub free_cache: Vec<usize>,
  pub max_param_size: usize,
}

impl Generator {
  pub fn new(i: Vec<Node>, globals: Vec<Variable>, stack_frames: Vec<StackFrame>, max_param_size: usize) -> Self {
    Self {
      base: Processor::new(i, Box::new(|_, _| false) , Box::new(|_| 0), Box::new(|_| PathBuf::new())), 
      sections: Sections::default(),
      indent_depth: 0,
      used_registers: Vec::new(),
      functions: Vec::new(),
      globals: globals,
      vars: HashMap::new(),
      stack_frames: stack_frames,
      selected_stack_frame: -1,
      free_cache: Vec::new(),
      max_param_size: max_param_size
    }
  }

  pub fn generate_label(&self) -> String {
    let temp = unsafe { LABEL_ID };
    unsafe {
      LABEL_ID += 1;
    };
    format!("L_{}", temp)
  }

  fn compile_literal(&mut self, lit: &Literal) -> MemoryLocation {
    match lit {
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
        let simd = self.get_ret_simd(fsize);
        self.r#move(&simd, &format!("[{}]", lbl), &lit.get_type());
        MemoryLocation::Register(simd)
      }
    }
  }

  fn compile_lvalue(&mut self, expr: &Expression) -> MemoryLocation {
    match &expr.kind {
      ExprKind::Variable(id) => {
        self.vars.get(id).unwrap().location.clone()
      },
      ExprKind::Dereference(expr) => {
        let loc = self.compile_expr(expr);
        let (reg, reg_id) = self.get_unused_register(expr.return_type.get_size(), expr.return_type.is_float());
        self.r#move(&reg, &loc.get(), &expr.return_type);
        self.free_cache.push(reg_id);
        MemoryLocation::Data(reg)
      },
      ExprKind::Index { base, index } => {
        let base_loc = self.compile_expr(base);
        let index_loc = self.compile_expr(index);
        let (base_reg, id1) = self.get_unused_register(base.return_type.get_size(), base.return_type.is_float());
        let (index_reg, id2) = self.get_unused_register(index.return_type.get_size(), index.return_type.is_float());
        self.mov(&base_reg, &base_loc.get());
        self.mov(&index_reg, &index_loc.get());
        self.free_cache.push(id1);
        self.free_cache.push(id2);
        MemoryLocation::Data(format!("{}+{}*{}", base_reg, index_reg, expr.return_type.get_size()))
      },
      ExprKind::FieldAccess { field, .. } => {
        self.vars.get(&field.id).unwrap().location.clone()
      },
      _ => {
        self.base.error(&format!("Invalid lvalue {}", expr))
      }
    }
  }

  fn compile_expr(&mut self, expr: &Expression) -> MemoryLocation {
    match &expr.kind {
      ExprKind::Literal(lit) => self.compile_literal(lit),
      ExprKind::Variable(id) => {
        self.vars.get(id).unwrap().clone().location
      },
      ExprKind::SizeOf(t) => {
        let num = t.get_size().to_string();
        let ret = self.get_ret_reg(get_configs().biggest_size);
        self.mov(&ret, &num);
        MemoryLocation::Register(ret)
      },
      ExprKind::Reference(ex) => {
        let reg = self.get_ret_reg(expr.return_type.get_size());
        let location = self.compile_expr(ex);
        self.lea(&reg, &location.get());
        MemoryLocation::Register(reg)
      },
      ExprKind::Dereference(ex) => {
        let isfloat = ex.return_type.is_float();
        let (reg, regid) = self.get_unused_register(ex.return_type.get_size(), isfloat);
        
        let loc = self.compile_expr(ex);
        self.r#move(&reg, &loc.get(), &ex.return_type);
        self.free_cache.push(regid);
        let temp = MemoryLocation::Data(reg);
        let ret = self.get_return(ex.return_type.get_size(), isfloat);
        self.r#move(&ret, &temp.get(), &ex.return_type);
        MemoryLocation::Register(ret)
      },
      ExprKind::Assignment { left, right } => {
        let (r_reg, r_id) = self.get_unused_register(right.return_type.get_size(), right.return_type.is_float());
        let right_loc = self.compile_expr(right);
        let left_loc = self.compile_lvalue(left);
        self.r#move(&r_reg, &right_loc.get(), &right.return_type);
        self.r#move(&left_loc.get(), &r_reg, &right.return_type);
        self.free_cache.push(r_id);
        left_loc
      },
      ExprKind::FncCall { id, args } => {
        let configs = get_configs();

        

        unreachable!()
      }
      _ => unimplemented!()
    }
  }

  fn evaluate(&self, expr: &Expression) -> Literal {
    match &expr.kind {
      ExprKind::Literal(lit) => lit.clone(),
      ExprKind::Cast { base, ..} => self.evaluate(&base),
      ExprKind::Variable(var) => {
        let context = self.vars.get(&var).unwrap();
        if let Some(expr) = &context.expr {
          self.evaluate(expr)
        } else {
          self.base.error(&format!("Variable of id {} has no value", var));
        }
      },
      ExprKind::SizeOf(t) => Literal::Integer(t.get_size() as u64),
      _ => self.base.error(&format!("Expression {} is not evaluable at compiletime", expr))
    }
  }

  fn compile_one(&mut self, node: &Node, total_allocated: isize) {
    match node {
      Node::Scope(s) | Node::Packet(s) => {
        s.iter().for_each(|node| {
          self.compile_one(node, total_allocated);
        });
      },
      Node::FncDecl(fnc) => {
        self.functions.push(fnc.clone());
        if let Some(body) = &fnc.body {
          self.create_function(&fnc.name, |this, total_alloc| {
            this.compile_one(&body, total_alloc);
          }, &get_configs().default_abi, fnc);
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
            self.vars.insert(var.id, VarContext { expr: Some(expr.clone()), r#type: var.r#type.clone(), location: MemoryLocation::Data(var.name.clone())});
          } else {
            self.uninit_alloc(&var.name, var.r#type.get_size());
            self.vars.insert(var.id, VarContext { expr: None, r#type: var.r#type.clone(), location: MemoryLocation::Data(var.name.clone()) });
          }
        } else {
          let val = if let Some(expr) = expr {
            self.compile_expr(expr).get()
          } else {
            String::from("0")
          };
          let loc = self.alloc_var(var.id, &val);
          self.vars.insert(var.id, VarContext { expr: expr.clone(), location: loc,  r#type: var.r#type.clone() });
          self.free_cache();
        }
      },
      Node::Return(ex) => {
        let isfloat = ex.return_type.is_float();
        let ret = self.get_return(ex.return_type.get_size(), ex.return_type.is_float());
        if ex.is_evaluable(&self.globals) {
          let lit = self.evaluate(ex);
          let location = self.compile_literal(&lit);
          self.r#move(&ret, &location.get(), &ex.return_type);
          let configs = get_configs();
          self.add(&configs.registers.stack_pointer[0], &format!("{}", total_allocated.abs()));
          self.mov(&configs.registers.stack_pointer[0], &configs.registers.base_pointer[0]);
          self.pop(&configs.registers.base_pointer[0]);
          self.ret();
        } else {
          let (temp, tempid) = self.get_unused_register(ex.return_type.get_size(), isfloat);
          let location = self.compile_expr(ex);
          self.r#move(&temp, &location.get(), &ex.return_type);
          self.r#move(&ret, &ret, &ex.return_type);
          self.free_register(tempid);
          let configs = get_configs();
          self.add(&configs.registers.stack_pointer[0], &format!("{}", total_allocated.abs()));
          self.mov(&configs.registers.stack_pointer[0], &configs.registers.base_pointer[0]);
          self.pop(&configs.registers.base_pointer[0]);
          self.ret();
        }
        self.free_cache();
      },
      Node::Expr(expr) => {
        self.compile_expr(expr);
      },
      Node::Assembly(code) => {
        let mut buffer = String::new();
        code.iter().for_each(|chunk| match chunk {
          AssemblyChunk::Original(s) => buffer.push_str(&s),
          AssemblyChunk::Var(id) => buffer.push_str(&self.vars.get(id).unwrap().location.get()),
        });
        buffer.lines().map(|ln| format!("{}{}", "\t".repeat(self.indent_depth), ln))
          .for_each(|l| { self.sections.text.push_str(&format!("{}\n", l)); });
      }
      //TODO THINK ABOUT CALLING CONVENTIONS AND IMPLEMENT IT IN FUNCTION PROLOGUE
      _ => unimplemented!()
    }
  }

  pub fn compile(&mut self) -> String {
    while self.base.has_peek() {
      let node = self.base.consume();
      self.compile_one(&node, 0);
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
