use crate::{constants::get_configs, generator::generator::Generator};


impl Generator {
  pub fn mov(&mut self, dst: &str, src: &str) {
    self.sections.text.push_str(&format!("{}mov {}, {}\n", "\t".repeat(self.indent_depth), dst, src));
  }

  pub fn add(&mut self, dst: &str, src: &str) {
    self.sections.text.push_str(&format!("{}add {}, {}\n", "\t".repeat(self.indent_depth), dst, src));
  }
  
  pub fn sub(&mut self, dst: &str, src: &str) {
    self.sections.text.push_str(&format!("{}sub {}, {}\n", "\t".repeat(self.indent_depth), dst, src));
  }

  pub fn cmp(&mut self, a: &str, b: &str) {
    self.sections.text.push_str(&format!("{}cmp {}, {}\n", "\t".repeat(self.indent_depth), a, b));
  }

  pub fn jmp(&mut self, lbl: &str) {
    self.sections.text.push_str(&format!("{}jmp {}\n", "\t".repeat(self.indent_depth), lbl));
  }

  pub fn jz(&mut self, lbl: &str) {
    self.sections.text.push_str(&format!("{}jz {}\n", "\t".repeat(self.indent_depth), lbl));
  }

  pub fn jnz(&mut self, lbl: &str) {
    self.sections.text.push_str(&format!("{}jnz {}\n", "\t".repeat(self.indent_depth), lbl));
  }

  pub fn jg(&mut self, lbl: &str) {
    self.sections.text.push_str(&format!("{}jg {}\n", "\t".repeat(self.indent_depth), lbl));
  }

  pub fn jl(&mut self, lbl: &str) {
    self.sections.text.push_str(&format!("{}jl {}\n", "\t".repeat(self.indent_depth), lbl));
  }

  pub fn jle(&mut self, lbl: &str) {
    self.sections.text.push_str(&format!("{}jle {}\n", "\t".repeat(self.indent_depth), lbl));
  }
  
  pub fn jge(&mut self, lbl: &str) {
    self.sections.text.push_str(&format!("{}jge {}\n", "\t".repeat(self.indent_depth), lbl));
  }

  pub fn call(&mut self, lbl: &str) {
    self.sections.text.push_str(&format!("{}call {}\n", "\t".repeat(self.indent_depth), lbl));
  }

  pub fn ret(&mut self) {
    self.sections.text.push_str(&format!("{}ret\n", "\t".repeat(self.indent_depth)));
  }

  pub fn push(&mut self, item: &str) {
    self.sections.text.push_str(&format!("{}push {}\n", "\t".repeat(self.indent_depth), item));
  }

  pub fn pop(&mut self, loc: &str) {
    self.sections.text.push_str(&format!("{}pop {}\n", "\t".repeat(self.indent_depth), loc));
  }

  pub fn create_function(&mut self, name: &str, f: impl FnOnce(&mut Generator)) {
    let configs = get_configs();
    self.sections.text.push_str(&format!("{}:\n", name));
    self.indent_depth += 1;
    self.push(&configs.registers.base_pointer[0]);
    self.mov(&configs.registers.base_pointer[0], &configs.registers.stack_pointer[0]);
    f(self);
    self.mov(&configs.registers.stack_pointer[0], &configs.registers.base_pointer[0]);
    self.pop(&configs.registers.base_pointer[0]);
    self.ret();
    self.indent_depth -= 1;
  }

  fn get_allocation_instruction(&self, size: usize) -> String {
    if size == 8 {
      "dq"
    } else if size == 4 {
      "dd"
    } else if size == 2 {
      "dw"
    } else if size == 1 {
      "db"
    } else {
      self.base.error(&format!("Invalid .data alloc size {}", size))
    }.into()
  }

  fn get_uninit_alloc_ins(&self, size: usize) -> (String, usize) {
    if size % 8 == 0 {
      (String::from("resq"), size / 8)
    } else if size % 4 == 0 {
      (String::from("resd"), size / 4)
    } else if size % 2 == 0 {
      (String::from("resw"), size / 2)
    } else {
      (String::from("resb"), size)
    }
  }

  pub fn init_alloc<'a>(&mut self, name: &'a str, size: usize, value: &str) -> &'a str {
    self.sections.data.push_str(&format!("{}{}: {} {}\n", "\t".repeat(self.indent_depth), name, self.get_allocation_instruction(size), value));
    name
  }

  pub fn const_alloc<'a>(&mut self, name: &'a str, size: usize, value: &str) -> &'a str {
    self.sections.read_only.push_str(&format!("{}{}: {} {}\n", "\t".repeat(self.indent_depth), name, self.get_allocation_instruction(size), value));
    name
  }

  pub fn uninit_alloc<'a>(&mut self, name: &'a str, size: usize) -> &'a str {
    let (ins, sz) = self.get_uninit_alloc_ins(size);
    self.sections.bss.push_str(&format!("{}{}: {} {}\n", "\t".repeat(self.indent_depth), name, ins, sz));
    name
  }

  pub fn alloc_str<'a>(&mut self, name: &'a str, s: &str) -> &'a str {
    self.sections.data.push_str(&format!("{}{}: db \"{}\"\n", "\t".repeat(self.indent_depth), name, s));
    name
  }

  pub fn alloc_str_const<'a>(&mut self, name: &'a str, s: &str) -> &'a str {
    self.sections.read_only.push_str(&format!("{}{}: db \"{}\"\n", "\t".repeat(self.indent_depth), name, s));
    name
  }

  pub fn get_unused_register(&mut self, size: usize) -> String {
    let configs = get_configs();
    let index: usize = (configs.biggest_size / size).ilog2() as usize;
    for (i, reg) in configs.registers.basic.iter().enumerate() {
      if self.used_registers.contains(&i) {
        continue;
      }
      self.used_registers.push(i);
      return reg[index].clone()
    }
    self.base.error(&format!("Cannot find unused register of size {}", size))
  }

  pub fn free_register(&mut self, reg: String) {
    let configs = get_configs();
    if let Some(found) = configs.registers.basic.iter().enumerate().find(|(_, v)| v.contains(&reg)) {
      self.used_registers.remove(found.0);
    }
  }

  pub fn get_ret_reg(&self, size: usize) -> String {
    let configs = get_configs();
    let index: usize = (configs.biggest_size / size).ilog2() as usize;
    configs.registers.return_register.get(index)
      .unwrap_or_else(|| self.base.error("Cannot get return register")).clone()
  }
  
}
