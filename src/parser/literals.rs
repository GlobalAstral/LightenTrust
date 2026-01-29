use std::error::Error;


pub enum Literal {
  Integer(u64),
  Float(f64),
  Char(u8),
  String(String)
}

impl Literal {
  pub fn from(s: &str) -> Result<Self, Box<dyn Error>> {
    if s.starts_with('\'') {
      Ok(Literal::Char(s.bytes().nth(1).unwrap()))
    } else if s.starts_with('"') {
      Ok(Literal::String(s[1..s.len()-1].to_string()))
    } else if let Some(hex) = s.strip_prefix("0x") {
      let num = u64::from_str_radix(hex, 16)?;
      Ok(Literal::Integer(num))
    } else if s.chars().filter(|&x| x == '.').count() == 1 {
      let num = s.parse::<f64>()?;
      Ok(Literal::Float(num))
    } else if s.chars().filter(|&x| x == '.').count() == 0 {
      let num = s.parse::<u64>()?;
      Ok(Literal::Integer(num))
    } else {
      Err(format!("Invalid literal \"{}\"", s).into())
    }
  }
}
