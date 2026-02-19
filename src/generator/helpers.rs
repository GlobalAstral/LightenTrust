use crate::generator::generator::Generator;


impl Generator {
  pub fn mov(&mut self, dst: &str, src: &str) {
    self.sections.text.push_str(&format!("mov {}, {}", dst, src));
  }

  pub fn add(&mut self, dst: &str, src: &str) {
    self.sections.text.push_str(&format!("add {}, {}", dst, src));
  }
  
  pub fn sub(&mut self, dst: &str, src: &str) {
    self.sections.text.push_str(&format!("sub {}, {}", dst, src));
  }

  pub fn cmp(&mut self, a: &str, b: &str) {
    self.sections.text.push_str(&format!("cmp {}, {}", a, b));
  }

  pub fn jmp(&mut self, lbl: &str) {
    self.sections.text.push_str(&format!("jmp {}", lbl));
  }

  pub fn jz(&mut self, lbl: &str) {
    self.sections.text.push_str(&format!("jz {}", lbl));
  }

  pub fn jnz(&mut self, lbl: &str) {
    self.sections.text.push_str(&format!("jnz {}", lbl));
  }

  pub fn jg(&mut self, lbl: &str) {
    self.sections.text.push_str(&format!("jg {}", lbl));
  }

  pub fn jl(&mut self, lbl: &str) {
    self.sections.text.push_str(&format!("jl {}", lbl));
  }

  pub fn jle(&mut self, lbl: &str) {
    self.sections.text.push_str(&format!("jle {}", lbl));
  }
  
  pub fn jge(&mut self, lbl: &str) {
    self.sections.text.push_str(&format!("jge {}", lbl));
  }

  pub fn call(&mut self, lbl: &str) {
    self.sections.text.push_str(&format!("call {}", lbl));
  }

  pub fn ret(&mut self) {
    self.sections.text.push_str("ret");
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
      (String::from("dq"), size / 8)
    } else if size % 4 == 0 {
      (String::from("dd"), size / 4)
    } else if size % 2 == 0 {
      (String::from("dw"), size / 2)
    } else {
      (String::from("db"), size)
    }
  }

  pub fn init_alloc(&mut self, name: &str, size: usize, value: &str) {
    self.sections.data.push_str(&format!("{}: {} {}", name, self.get_allocation_instruction(size), value));
  }

  pub fn const_alloc(&mut self, name: &str, size: usize, value: &str) {
    self.sections.read_only.push_str(&format!("{}: {} {}", name, self.get_allocation_instruction(size), value));
  }

  pub fn uninit_alloc(&mut self, name: &str, size: usize) {
    let (ins, sz) = self.get_uninit_alloc_ins(size);
    self.sections.bss.push_str(&format!("{}: {} {}", name, ins, sz));
  }

}
