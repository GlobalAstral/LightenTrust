use std::{collections::HashMap, default, path::PathBuf};

use crate::{constants::get_configs, parser::{nodes::{Fnc, Node}, utils::Processor}};

#[derive(Debug, Clone)]
pub struct StackFrame {
  pub next_ofs: isize,
  pub locals: HashMap<u64, isize>
}

impl StackFrame {
  pub fn new() -> Self {
    Self { next_ofs: 0, locals: HashMap::new() }
  }
}

pub struct Scanner {
  pub base: Processor<Node>,
  pub functions: Vec<Fnc>,
  pub stack_frames: Vec<StackFrame>,
  pub selected_stack_frame: isize,
  max_function_param_size: usize,
}

impl Scanner {
  pub fn new(i: Vec<Node>) -> Scanner {
    Scanner {
      base: Processor::new(i, Box::new(|_, _| false) , Box::new(|_| 0), Box::new(|_| PathBuf::new())), 
      functions: Vec::new(), 
      stack_frames: Vec::new(),
      selected_stack_frame: -1,
      max_function_param_size: 0,
    }
  }

  pub fn calculate(&mut self, node: &Node) {
    match node {
      Node::Scope(scope) | Node::Packet(scope) => {
        scope.iter().for_each(|node| self.calculate(node));
      },
      Node::FncDecl(fnc) => {
        let configs = get_configs();

        let abi = configs.abis.iter().find(|t| t.name == configs.default_abi)
          .unwrap_or_else(|| self.base.error(&format!("ABI '{}' not found", configs.default_abi)));

        let total_size = fnc.arguments.iter().enumerate().fold(0, |acc, (index, var)| {
          if var.r#type.is_float() && abi.parameter_simds.iter().find(|(i, _)| *i == index).is_none()  {
            return acc + var.r#type.get_size()
          }
          if abi.parameter_registers.iter().find(|(i, _)| *i == index).is_none() {
            return acc + var.r#type.get_size()
          }
          acc
        });

        if total_size > self.max_function_param_size {
          self.max_function_param_size = total_size;
        }
        self.stack_frames.push(StackFrame::new());
        if let Some(body) = &fnc.body {
          self.selected_stack_frame += 1;
          self.calculate(&body);
          self.selected_stack_frame -= 1;
        }
      },
      Node::ExternFnc(fnc) => {
        let configs = get_configs();

        let abi = configs.abis.iter().find(|t| t.name == configs.default_abi)
          .unwrap_or_else(|| self.base.error(&format!("ABI '{}' not found", configs.default_abi)));

        let total_size = fnc.arguments.iter().enumerate().fold(0, |acc, (index, var)| {
          if var.r#type.is_float() && abi.parameter_simds.iter().find(|(i, _)| *i == index).is_none()  {
            return acc + var.r#type.get_size()
          }
          if abi.parameter_registers.iter().find(|(i, _)| *i == index).is_none() {
            return acc + var.r#type.get_size()
          }
          acc
        });

        if total_size > self.max_function_param_size {
          self.max_function_param_size = total_size;
        }
      },
      Node::VariableDecl { var, .. } => {
        if self.selected_stack_frame < 0 {
          self.base.error("No stackframe found");
        }
        if var.global {
          return;
        }
        if let Some(frame) = self.stack_frames.get_mut(self.selected_stack_frame as usize) {
          frame.next_ofs += var.r#type.get_size() as isize;
          frame.locals.insert(var.id, frame.next_ofs);
        }
      },
      _ => { }
    }
  }

  pub fn calc_all(&mut self) -> (usize, Vec<StackFrame>) {
    let configs = get_configs();

    let abi = configs.abis.iter().find(|t| t.name == configs.default_abi)
      .unwrap_or_else(|| self.base.error(&format!("ABI '{}' not found", configs.default_abi)));

    while self.base.has_peek() {
      let node = self.base.consume();
      self.calculate(&node);
    }

    return (self.max_function_param_size + abi.shadow_space, self.stack_frames.clone());
  }
}
