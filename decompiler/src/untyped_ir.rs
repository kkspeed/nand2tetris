use std::fmt;
use std::fmt::Display;

use parser::{VmCommand, Segment};

#[derive(Debug, Clone)]
pub enum UnTypedIR {
    FuncDef(String, Vec<UnTypedIR>),
    ConstInt(i32),
    ConstString(String),
    Var(String),
    Unary(String, Box<UnTypedIR>),
    Binary(String, Box<UnTypedIR>, Box<UnTypedIR>),
    Call(String, Vec<UnTypedIR>),
    Assign(Box<UnTypedIR>, Box<UnTypedIR>),
    Return(Box<UnTypedIR>),
    If(Box<UnTypedIR>, Vec<UnTypedIR>, Vec<UnTypedIR>, Vec<UnTypedIR>),
    While(Box<UnTypedIR>, Vec<UnTypedIR>, Vec<UnTypedIR>),
    ArrayOffset(Box<UnTypedIR>, Box<UnTypedIR>),
}

impl UnTypedIR {
    pub fn is_const_int(&self) -> bool {
        match self {
            &UnTypedIR::ConstInt(_) => true,
            _ => false,
        }
    }

    pub fn is_const_string(&self) -> bool {
        match self {
            &UnTypedIR::ConstString(_) => true,
            _ => false,
        }
    }

    pub fn int(&self) -> i32 {
        match self {
            &UnTypedIR::ConstInt(i) => i,
            _ => panic!("{} is not int!", self)
        }
    }

    pub fn str(&self) -> &str {
        match self {
            &UnTypedIR::ConstString(ref s) => s,
            _ => panic!("{} is not string!", self),
        }
    }

    pub fn is_const_funcall(&self, func: &str, n: usize) -> bool {
        match self {
            &UnTypedIR::Call(ref s, ref args) => {
                s == func && args.len() == n && args.iter().all(|a| a.is_const_int() || a.is_const_string())
            }
            _ => false,
        }
    }

    pub fn to_array_offset(self) -> Self {
        match self {
            UnTypedIR::Binary(op, e1, e2) => {
                if op == "+" {
                    UnTypedIR::ArrayOffset(e1, e2)
                } else {
                    panic!("array offset only contains + sign")
                }
            }
            UnTypedIR::Var(v) => UnTypedIR::ArrayOffset(Box::new(UnTypedIR::Var(v)), Box::new(UnTypedIR::ConstInt(0))),
            UnTypedIR::Assign(_, e) => e.to_array_offset(),
            _ => panic!("cannot get array offset from: {:?}", self),
        }
    }

    pub fn is_assigned_to(&self, var: &str) -> bool {
        match self {
            &UnTypedIR::Var(ref s) => s == var,
            &UnTypedIR::Assign(ref s, _) =>  {
                match s.as_ref() {
                    &UnTypedIR::Var(ref v) => v == var,
                    _ => false,
                }
            }
            _ => false,
        }
    }

    pub fn has_use(&self, var: &str) -> bool {
        match self {
            &UnTypedIR::Call(_, ref args) => args.iter().any(|i| i.has_use(var)),
            &UnTypedIR::Assign(ref s, ref expr) => s.has_use(var) || expr.has_use(var),
            &UnTypedIR::Binary(_, ref e1, ref e2) => e1.has_use(var) || e2.has_use(var),
            &UnTypedIR::ConstInt(_) => false,
            &UnTypedIR::ConstString(_) => false,
            &UnTypedIR::FuncDef(_, ref exprs) => exprs.iter().any(|i| i.has_use(var)),
            &UnTypedIR::Unary(_, ref e) => e.has_use(var),
            &UnTypedIR::Var(ref v) => v == var,
            &UnTypedIR::If(ref c, ref ts, ref fs, ref conts) => c.has_use(var) || ts.iter().any(|i| i.has_use(var)) || fs.iter().any(|i| i.has_use(var)) || conts.iter().any(|i| i.has_use(var)),
            &UnTypedIR::While(ref c, ref bs, ref conts) => c.has_use(var) || bs.iter().any(|i| i.has_use(var)) || conts.iter().any(|i| i.has_use(var)),
            &UnTypedIR::Return(ref e) => e.has_use(var),
            &UnTypedIR::ArrayOffset(ref base, ref offset) => base.has_use(var) || offset.has_use(var),
        }
    }

    pub fn replace_var(self, var: &str, exp: &UnTypedIR) -> Self {
        match self {
            UnTypedIR::Call(s, args) => UnTypedIR::Call(s, args.into_iter().map(|i| i.replace_var(var, exp)).collect()),
            UnTypedIR::Assign(v, expr) => UnTypedIR::Assign(Box::new(v.replace_var(var, exp)), Box::new(expr.replace_var(var, exp))),
            UnTypedIR::Binary(op, e1, e2) => UnTypedIR::Binary(op, Box::new(e1.replace_var(var, exp)), Box::new(e2.replace_var(var, exp))),
            UnTypedIR::FuncDef(func, exprs) => UnTypedIR::FuncDef(func, exprs.into_iter().map(|i| i.replace_var(var, exp)).collect()),
            UnTypedIR::Unary(op, e) => UnTypedIR::Unary(op, Box::new(e.replace_var(var, exp))),
            UnTypedIR::Var(v) => if v == var { exp.clone() } else { UnTypedIR::Var(v) },
            UnTypedIR::Return(e) => UnTypedIR::Return(Box::new(e.replace_var(var, exp))),
            UnTypedIR::If(c,  ts,  fs, conts) => UnTypedIR::If(Box::new(c.replace_var(var, exp)), ts.into_iter().map(|i| i.replace_var(var, exp)).collect(), fs.into_iter().map(|i| i.replace_var(var, exp)).collect(), conts.into_iter().map(|i| i.replace_var(var, exp)).collect()),
            UnTypedIR::While(c, bs, conts) => UnTypedIR::While(Box::new(c.replace_var(var, exp)), bs.into_iter().map(|i| i.replace_var(var, exp)).collect(), conts.into_iter().map(|i| i.replace_var(var, exp)).collect()),
            UnTypedIR::ArrayOffset(base, offset) => UnTypedIR::ArrayOffset(base, Box::new(offset.replace_var(var, exp))),
            i => i,
        }
    }

    pub fn reconstruct_const_string(self) -> Self {
        match self {
            UnTypedIR::FuncDef(s, irs) => {
                UnTypedIR::FuncDef(s, irs.into_iter().map(|i| i.reconstruct_const_string()).collect())
            }
            UnTypedIR::Unary(s, ir) => UnTypedIR::Unary(s, Box::new(ir.reconstruct_const_string())),
            UnTypedIR::Binary(s, i1, i2) => UnTypedIR::Binary(s, Box::new(i1.reconstruct_const_string()), Box::new(i2.reconstruct_const_string())),
            UnTypedIR::Call(s, irs) => {
                if s == "String.appendChar" && irs.len() == 2 {
                    let reconstructed: Vec<UnTypedIR> = irs.into_iter().map(|i| i.reconstruct_const_string()).collect();
                    if reconstructed[1].is_const_int() && reconstructed[0].is_const_string() {
                        return UnTypedIR::ConstString(format!("{}{}", reconstructed[0].str(), reconstructed[1].int() as u8 as char));
                    }
                    if reconstructed[0].is_const_funcall("String.new", 1)  {
                        if reconstructed[1].is_const_int() {
                            return UnTypedIR::ConstString(format!("{}", reconstructed[1].int() as u8 as char));
                        }
                    }
                    UnTypedIR::Call(s, reconstructed)
                } else {
                    UnTypedIR::Call(s, irs.into_iter().map(|i| i.reconstruct_const_string()).collect())
                }
            }
            UnTypedIR::Assign(v, ir) => UnTypedIR::Assign(v, Box::new(ir.reconstruct_const_string())),
            UnTypedIR::Return(ir) => UnTypedIR::Return(Box::new(ir.reconstruct_const_string())),
            UnTypedIR::If(cond, true_exprs, false_exprs, conts) => 
                UnTypedIR::If(Box::new(cond.reconstruct_const_string()), 
                    true_exprs.into_iter().map(|i| i.reconstruct_const_string()).collect(), 
                    false_exprs.into_iter().map(|i| i.reconstruct_const_string()).collect(),
                    conts.into_iter().map(|i| i.reconstruct_const_string()).collect()),
            UnTypedIR::While(cond, body, conts) => 
                UnTypedIR::While(Box::new(cond.reconstruct_const_string()),
                body.into_iter().map(|i| i.reconstruct_const_string()).collect(),
                conts.into_iter().map(|i| i.reconstruct_const_string()).collect()),
            x => x
        }
    }
}

impl Display for UnTypedIR {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            &UnTypedIR::FuncDef(ref func, ref body) => {
                write!(f, "function {}(...) {{\n", func)?;
                for i in body.iter() {
                    write!(f, "{}\n", i)?;
                }
                write!(f, "}}\n")
            } 
            &UnTypedIR::ConstInt(i) => write!(f, "{}", i),
            &UnTypedIR::ConstString(ref s) => write!(f, "\"{}\"", s),
            &UnTypedIR::Var(ref s) => write!(f, "{}", s),
            &UnTypedIR::Unary(ref op, ref e) => write!(f, "{}({})", op, e),
            &UnTypedIR::Binary(ref op, ref e1, ref e2) => write!(f, "{} {} {}", e1, op, e2),
            &UnTypedIR::Call(ref func, ref args) => {
                write!(f, "{}(", func)?;
                for arg in args.iter().take(1) {
                    write!(f, "{}", arg)?;
                }
                for arg in args.iter().skip(1) {
                    write!(f, ", {}", arg)?;
                }
                write!(f, ")")
            }
            &UnTypedIR::Assign(ref e1, ref e2) => write!(f, "let {} = {};", e1, e2),
            &UnTypedIR::Return(ref e) => write!(f, "return({});", e),
            &UnTypedIR::If(ref e, ref taken, ref not_taken, ref cont) => {
                write!(f, "if ({}) {{\n", e)?;
                for s in taken.iter() {
                    write!(f, "{}\n", s)?;
                }
                if !not_taken.is_empty() {
                    write!(f, "}} else {{\n")?;
                    for s in not_taken.iter() {
                        write!(f, "{}\n", s)?;
                    }
                    write!(f, "}}\n")?;
                } else {
                    write!(f, "}}\n")?;
                }
                for s in cont.iter() {
                    write!(f, "{}\n", s)?;
                }
                Ok(())
            }
            &UnTypedIR::While(ref e, ref body, ref cont) => {
                write!(f, "while ({}) {{\n", e)?;
                for s in body.iter() {
                    write!(f, "{}\n", s)?;
                }
                write!(f, "}}\n")?;
                for s in cont.iter() {
                    write!(f, "{}\n", s)?;
                }
                Ok(())
            }
            &UnTypedIR::ArrayOffset(ref e1, ref e2) =>  write!(f, "{}[{}]", e1, e2)
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
                    stack.push(UnTypedIR::ConstInt(i));
                } else {
                    stack.push(UnTypedIR::Var(format!("{:?}_{}", seg, i)));
                }
            }
            &VmCommand::Pop(seg, i) => {
                let e = stack.pop().unwrap();
                if seg == Segment::CONST {
                    result.push(UnTypedIR::Assign(Box::new(UnTypedIR::Var(format!("{}", i))), Box::new(e)));
                } else {
                    result.push(UnTypedIR::Assign(Box::new(UnTypedIR::Var(format!("{:?}_{}", seg, i))), Box::new(e)));
                }
            }
            &VmCommand::Call(ref func, n) => {
                let mut args = Vec::new();
                for _ in 0..n {
                    let e = stack.pop().unwrap();
                    args.push(e);
                }
                args.reverse();
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