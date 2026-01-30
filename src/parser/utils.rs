use std::process::exit;


pub trait Throws<T> {
  fn or_throw(self, f: impl FnOnce()) -> T;
}

impl<T> Throws<T> for Option<T> {
  fn or_throw(self, f: impl FnOnce()) -> T {
    if self.is_some() {
      return self.unwrap()
    }
    f();
    exit(1);
  }
}
