use crate::{constants::get_configs, generator::generator::{Generator, MemoryLocation}, parser::{nodes::Fnc, types::Type}, scanner::scanner::StackFrame};


impl Generator {
  pub fn mov(&mut self, dst: &str, src: &str) {
    self.sections.text.push_str(&format!("{}mov {}, {}\n", "\t".repeat(self.indent_depth), dst, src));
  }

  pub fn movs(&mut self, dst: &str, src: &str, suffix: &str) {
    self.sections.text.push_str(&format!("{}mov{} {}, {}\n", "\t".repeat(self.indent_depth), suffix, dst, src));
  }

  pub fn r#move(&mut self, dst: &str, src: &str, r#type: &Type) {
    let configs = get_configs();
    if r#type.is_float() {
      let suffix = configs.floating_instructions_suffixes.iter().find(|(size, _)| {
        r#type.get_size() == *size
      }).unwrap_or_else(|| self.base.error(&format!("No float suffix exists for size {}", r#type.get_size())));
      self.movs(dst, src, &suffix.1);
    } else {
      self.mov(dst, src);
    }
  }

  pub fn lea(&mut self, dst: &str, src: &str) {
    self.sections.text.push_str(&format!("{}lea {}, {}\n", "\t".repeat(self.indent_depth), dst, src));
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

  pub fn create_function(&mut self, name: &str, f: impl Fn(&mut Generator, isize), abi: &str, fnc: &Fnc) {
    let configs = get_configs();
    self.selected_stack_frame += 1;

    let abi = configs.abis.iter().find(|cc| cc.name == abi)
    .unwrap_or_else(|| self.base.error(&format!("ABI {} does not exist", abi)));

    let next_ofs = self.get_stackframe().next_ofs;

    let total_alloc = if next_ofs % abi.stack_align as isize == 0 { next_ofs } else { 
      next_ofs + (abi.stack_align as isize - next_ofs % abi.stack_align as isize) 
    };
    
    self.sections.text.push_str(&format!("{}:\n", name));
    self.indent_depth += 1;
    
    self.push(&configs.registers.base_pointer[0]);
    self.mov(&configs.registers.base_pointer[0], &configs.registers.stack_pointer[0]);
    self.sub(&configs.registers.stack_pointer[0], &format!("{}", total_alloc.abs()));
    
    f(self, total_alloc);

    self.add(&configs.registers.stack_pointer[0], &format!("{}", total_alloc.abs()));
    self.mov(&configs.registers.stack_pointer[0], &configs.registers.base_pointer[0]);
    self.pop(&configs.registers.base_pointer[0]);
    self.ret();
    
    self.stack_frames.remove(self.selected_stack_frame as usize);
    self.selected_stack_frame -= 1;
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

  pub fn get_unused_register(&mut self, size: usize, simd: bool) -> (String, usize) {
    let configs = get_configs();
    let biggest = if simd {configs.biggest_simd} else {configs.biggest_size};
    let index: usize = (biggest / size).ilog2() as usize;
    let mut search = configs.registers.basic.clone();
    search.append(&mut configs.registers.simds.clone());
    for (i, reg) in search.iter().enumerate().skip(if simd { configs.registers.basic.len() } else {0}) {
      if self.used_registers.contains(&i) {
        continue;
      }
      self.used_registers.push(i);
      let temp = index.min(reg.len()-1);
      return (reg.get(temp).unwrap().clone(), temp)
    }
    self.base.error(&format!("Cannot find unused register of size {}", size))
  }

  pub fn free_register(&mut self, reg: usize) {
    if reg < self.used_registers.len() {
      self.used_registers.remove(reg);
    }
  }

  pub fn get_ret_reg(&self, size: usize) -> String {
    let configs = get_configs();
    let index: usize = (configs.biggest_size / size).ilog2() as usize;
    configs.registers.return_register.get(index.min(configs.registers.return_register.len()-1))
      .unwrap_or_else(|| self.base.error("Cannot get return register")).clone()
  }

  pub fn get_ret_simd(&self, size: usize) -> String {
    let configs = get_configs();
    let index: usize = (configs.biggest_simd / size).ilog2() as usize;
    configs.registers.return_simd.get(index.min(configs.registers.return_simd.len()-1))
      .unwrap_or_else(|| self.base.error("Cannot get return simd")).clone()
  }

  pub fn get_return(&self, size: usize, simd: bool) -> String {
    if simd {
      self.get_ret_simd(size)
    } else {
      self.get_ret_reg(size)
    }
  }

  fn get_stackframe(&mut self) -> &mut StackFrame {
    if self.selected_stack_frame < 0 {
      self.base.error("No stack-frame exists");
    }
    self.stack_frames.get_mut(self.selected_stack_frame as usize).unwrap()
  }

  pub fn alloc_var(&mut self, id: u64, value: &str) -> MemoryLocation {
    let frame = self.get_stackframe();
    
    let (_, offset) = frame.locals.iter().find(|(i, _)| id == **i).unwrap();
    
    let location = MemoryLocation::Stack(-*offset as isize);
    
    self.mov(&location.get(), value);
    return location;
  }

  pub fn free_cache(&mut self) {
    let temp = std::mem::take(&mut self.free_cache);
    for ele in &temp {
      self.free_register(*ele);
    }
    self.free_cache = temp;
    self.free_cache.clear();
  }
}
