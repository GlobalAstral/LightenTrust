use std::{fmt::Display, process::exit};

pub struct Processor<T> where T: Clone + Display + Default {
  input: Vec<T>,
  pub peek: usize,
  equals_criteria: Box<dyn Fn(&T, &T) -> bool>,
  get_line: Box<dyn Fn(&T) -> usize>
}

impl<T> Processor<T> where T: Clone + Display + Default {
  pub fn new(i: Vec<T>, criteria: Box<dyn Fn(&T, &T) -> bool>, get_line: Box<dyn Fn(&T) -> usize>) -> Processor<T> {
    Self { input: i, peek: 0, equals_criteria: criteria, get_line }
  }

  pub fn has_peek(&self) -> bool {
    self.peek < self.input.len()
  }

  pub fn error(&self, msg: &str) -> ! {
    eprintln!("Error: {} at line: {}", msg, (self.get_line)(&self.peek_back()));
    exit(1)
  }

  pub fn peek(&self) -> T {
    self.input.get(self.peek).cloned().unwrap_or_default()
  }

  pub fn peek_equal(&self, i: T) -> bool {
    if self.has_peek() && (self.equals_criteria)(&self.peek(), &i) {
      return true
    }
    false
  }

  pub fn peek_back(&self) -> T {
    self.input.get(self.peek.saturating_sub(1)).cloned().unwrap_or_default()
  }

  pub fn consume(&mut self) -> T {
    if self.has_peek() {
      let temp = self.input.get(self.peek).cloned().unwrap();
      self.peek += 1;
      return temp
    }
    T::default()
  }
  
  pub fn tryconsume(&mut self, i: T) -> bool {
    if self.has_peek() {
      if (self.equals_criteria)(&self.peek(), &i) {
        self.consume();
        return true;
      }
    }
    return false;
  }

  pub fn require(&mut self, i: T) -> T {
    if self.has_peek() {
      if (self.equals_criteria)(&self.peek(), &i) {
        return self.consume();
      }
    }
    self.error(&format!("Expected '{}'", i));
  }

  pub fn switch<R>(&mut self, i: Vec<T>, f: impl FnOnce(&mut Self) -> R) -> R {
    let old = self.input.clone();
    let old_peek = self.peek;
    self.input = i;
    self.peek = 0;
    let ret = f(self);
    self.input = old;
    self.peek = old_peek;
    return ret;
  }
}
