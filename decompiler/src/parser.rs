use std::fmt;
use std::fmt::Display;
use std::io::{Read, BufReader};

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Segment {
    LCL,
    ARG,
    THIS,
    THAT,
    CONST,
    POINTER,
    STATIC,
    TEMP,
}

impl Segment {
    fn get_string(&self) -> &'static str {
        match self {
            &Segment::LCL => "LCL",
            &Segment::ARG => "ARG",
            &Segment::THIS => "THIS",
            &Segment::THAT => "THAT",
            _ => panic!("no representation for {:?}", self),
        }
    }

    fn from_string(s: &str) -> Self {
        match s {
            "local" => Segment::LCL,
            "argument" => Segment::ARG,
            "this" => Segment::THIS,
            "that" => Segment::THAT,
            "constant" => Segment::CONST,
            "pointer" => Segment::POINTER,
            "static" => Segment::STATIC,
            "temp" => Segment::TEMP,
            _ => panic!("unknown command: {}", s),
        }
    }
}

#[derive(Debug)]
pub enum VmCommand {
    Push(Segment, i32),
    Pop(Segment, i32),
    Add,
    Sub,
    Neg,
    Eq,
    Gt,
    Lt,
    And,
    Or,
    Not,
    Label(String),
    Goto(String),
    IfGoto(String),
    Call(String, i32),
    FunDef(String, i32),
    Return,
}

impl Display for VmCommand {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            &VmCommand::Push(seg, i) => write!(f, "push {:?} {}", seg, i),
            &VmCommand::Pop(seg, i) => write!(f, "pop {:?} {}", seg, i),
            &VmCommand::Label(ref s) => write!(f, "label {}", s),
            &VmCommand::Goto(ref s) => write!(f, "goto {}", s),
            &VmCommand::IfGoto(ref s) => write!(f, "if-goto {}", s),
            &VmCommand::FunDef(ref s, i) => write!(f, "function {} {}", s, i),
            &VmCommand::Call(ref s, i) => write!(f, "call {} {}", s, i),
            cmd => write!(f, "{:?}", cmd),
        }
    }
}

impl VmCommand {
    fn from_line(line: &str) -> Option<VmCommand> {
        let part = line.split("//").nth(0).unwrap();
        let trimmed = part.trim();
        if trimmed.starts_with("push") {
            let mut parts = trimmed.split(' ');
            parts.next();
            let segment = parts.next().unwrap();
            let offset = parts.next().unwrap().parse::<i32>().unwrap();
            Some(VmCommand::Push(Segment::from_string(segment), offset))
        } else if trimmed.starts_with("pop") {
            let mut parts = trimmed.split(' ');
            parts.next();
            let segment = parts.next().unwrap();
            let offset = parts.next().unwrap().parse::<i32>().unwrap();
            Some(VmCommand::Pop(Segment::from_string(segment), offset))
        } else if trimmed.starts_with("add") {
            Some(VmCommand::Add)
        } else if trimmed.starts_with("sub") {
            Some(VmCommand::Sub)
        } else if trimmed.starts_with("neg") {
            Some(VmCommand::Neg)
        } else if trimmed.starts_with("eq") {
            Some(VmCommand::Eq)
        } else if trimmed.starts_with("gt") {
            Some(VmCommand::Gt)
        } else if trimmed.starts_with("lt") {
            Some(VmCommand::Lt)
        } else if trimmed.starts_with("and") {
            Some(VmCommand::And)
        } else if trimmed.starts_with("or") {
            Some(VmCommand::Or)
        } else if trimmed.starts_with("not") {
            Some(VmCommand::Not)
        } else if trimmed.starts_with("label") {
            let label = trimmed.split(' ').nth(1).unwrap();
            Some(VmCommand::Label(label.into()))
        } else if trimmed.starts_with("goto") {
            let label = trimmed.split(' ').nth(1).unwrap();
            Some(VmCommand::Goto(label.into()))
        } else if trimmed.starts_with("if-goto") {
            let label = trimmed.split(' ').nth(1).unwrap();
            Some(VmCommand::IfGoto(label.into()))
        } else if trimmed.starts_with("call") {
            let mut parts = trimmed.split(' ');
            parts.next();
            let fun = parts.next().unwrap();
            let arg_count = parts.next().unwrap().parse::<i32>().unwrap();
            Some(VmCommand::Call(fun.into(), arg_count))
        } else if trimmed.starts_with("function") {
            let mut parts = trimmed.split(' ');
            parts.next();
            let fun = parts.next().unwrap();
            let var_count = parts.next().unwrap().parse::<i32>().unwrap();
            Some(VmCommand::FunDef(fun.into(), var_count))
        } else if trimmed.starts_with("return") {
            Some(VmCommand::Return)
        } else {
            None
        }
    }
}

pub fn vm_commands<R: Read>(r: R) -> Vec<VmCommand> {
    let mut reader = BufReader::new(r);
    let mut s = String::new();
    reader.read_to_string(&mut s).unwrap();
    s.lines().filter_map(|l| VmCommand::from_line(l)).collect()
}
