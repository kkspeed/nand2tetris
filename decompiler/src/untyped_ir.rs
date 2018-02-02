use std::fmt;
use std::fmt::Display;

use parser::{VmCommand, Segment};

pub enum UnTypedIR {
    ConstInt(i32),
    Var(String),
    Unary(String, Box<UnTypedIR>),
    Binary(String, Box<UnTypedIR>, Box<UnTypedIR>),
    Call(String, Vec<UnTypedIR>),
    Assign(String, Box<UnTypedIR>),
    Return(Box<UnTypedIR>),
}

impl Display for UnTypedIR {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            &UnTypedIR::ConstInt(i) => write!(f, "{}", i),
            &UnTypedIR::Var(ref s) => write!(f, "{}", s),
            &UnTypedIR::Unary(ref op, ref e) => write!(f, "{}({})", op, e),
            &UnTypedIR::Binary(ref op, ref e1, ref e2) => write!(f, "{} {} {}", e1, op, e2),
            &UnTypedIR::Call(ref func, ref args) => {
                write!(f, "{}(", func)?;
                for arg in args.iter() {
                    write!(f, "{}, ", arg)?;
                }
                write!(f, ")")
            }
            &UnTypedIR::Assign(ref e1, ref e2) => write!(f, "let {}={}", e1, e2),
            &UnTypedIR::Return(ref e) => write!(f, "return({})", e),
        }
    }
}

pub fn get_untyped_ir_from_vm_commands(cmds: &[VmCommand]) -> Vec<UnTypedIR> {
    let mut stack = Vec::new();
    let mut result = Vec::new();
    for cmd in cmds {
        match cmd {
            &VmCommand::Push(seg, i) => {
                if seg == Segment::CONST {
                    stack.push(UnTypedIR::Var(format!("{}", i)));
                } else {
                    stack.push(UnTypedIR::Var(format!("{:?}_{}", seg, i)));
                }
            }
            &VmCommand::Pop(seg, i) => {
                let e = stack.pop().unwrap();
                if seg == Segment::CONST {
                    result.push(UnTypedIR::Assign(format!("{}", i), Box::new(e)));
                } else {
                    result.push(UnTypedIR::Assign(format!("{:?}_{}", seg, i), Box::new(e)));
                }
            }
            &VmCommand::Call(ref func, n) => {
                let mut args = Vec::new();
                for _ in 0..n {
                    let e = stack.pop().unwrap();
                    args.push(e);
                }
                // ToDO: we need to reverse this.
                stack.push(UnTypedIR::Call(func.clone(), args));
            }
            &VmCommand::Neg => {
                let e = stack.pop().unwrap();
                stack.push(UnTypedIR::Unary("-".into(), Box::new(e)));
            }
            &VmCommand::Not => {
                let e = stack.pop().unwrap();
                stack.push(UnTypedIR::Unary("~".into(), Box::new(e)));
            }
            &VmCommand::Add => {
                let e1 = stack.pop().unwrap();
                let e2 = stack.pop().unwrap();
                stack.push(UnTypedIR::Binary("+".into(), Box::new(e1), Box::new(e2)));
            }
            &VmCommand::Lt => {
                let e1 = stack.pop().unwrap();
                let e2 = stack.pop().unwrap();
                stack.push(UnTypedIR::Binary(">".into(), Box::new(e1), Box::new(e2)));
            }
            &VmCommand::Gt => {
                let e1 = stack.pop().unwrap();
                let e2 = stack.pop().unwrap();
                stack.push(UnTypedIR::Binary("<".into(), Box::new(e1), Box::new(e2)));
            }
            &VmCommand::Sub => {
                let e1 = stack.pop().unwrap();
                let e2 = stack.pop().unwrap();
                stack.push(UnTypedIR::Binary("-".into(), Box::new(e2), Box::new(e1)));
            }
            &VmCommand::Eq => {
                let e1 = stack.pop().unwrap();
                let e2 = stack.pop().unwrap();
                stack.push(UnTypedIR::Binary("=".into(), Box::new(e1), Box::new(e2)));
            }
            &VmCommand::And => {
                let e1 = stack.pop().unwrap();
                let e2 = stack.pop().unwrap();
                stack.push(UnTypedIR::Binary("&".into(), Box::new(e1), Box::new(e2)));
            }
            &VmCommand::Or => {
                let e1 = stack.pop().unwrap();
                let e2 = stack.pop().unwrap();
                stack.push(UnTypedIR::Binary("|".into(), Box::new(e1), Box::new(e2)));
            }
            &VmCommand::Return => {
                let e = stack.pop().unwrap();
                result.push(UnTypedIR::Return(Box::new(e)));
            }
            &VmCommand::FunDef(_, _) => panic!("FunDef should not be handled here!"),
            &VmCommand::IfGoto(_) => panic!("IfGoto should not be handled here!"),
            &VmCommand::Label(_) => panic!("Label should not be handled here!"),
            &VmCommand::Goto(_) => panic!("Goto should not be handled here!"),
        }
    }
    if let Some(e) = stack.pop() {
        result.push(e);
    }
    result
}