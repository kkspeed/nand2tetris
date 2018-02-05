extern crate decompiler;

use decompiler::parser::{vm_commands};
use decompiler::decompiler::to_untyped_ir;

use std::fs;
use std::io::{BufWriter};

fn main() {
    let input = fs::File::open("test.vm").unwrap();
    let output = fs::File::create("test.dot").unwrap();
    let mut writer = BufWriter::new(output);
    let untyped_irs = to_untyped_ir(&mut vm_commands(input).into_iter());
    for c in untyped_irs {
        println!("{}", c.reconstruct_const_string());
    }
}
