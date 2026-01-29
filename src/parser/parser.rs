use std::{iter::Peekable, slice::Iter};

use crate::tokens::token::Token;


pub struct Parser {
  input: Peekable<std::vec::IntoIter<Token>>
}

impl Parser {
  pub fn new(i: Vec<Token>) -> Self {
    let temp: Peekable<std::vec::IntoIter<Token>> = i.into_iter().peekable();
    Self { input: temp }
  }
}
