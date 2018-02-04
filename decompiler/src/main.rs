extern crate decompiler;

use decompiler::parser::vm_commands;
use decompiler::decompiler::Graph;
use decompiler::untyped_ir::UnTypedIR;
use std::convert::From;

use std::fs;
use std::io::{BufWriter};

fn main() {
    let input = fs::File::open("test.vm").unwrap();
    let output = fs::File::create("test.dot").unwrap();
    let mut writer = BufWriter::new(output);
    let g: Graph<UnTypedIR> = From::from(Graph::build(vm_commands(input))); //.write_graphviz(&mut writer);
    g.write_graphviz(&mut writer);
    let rs = g.reconstruct_code();
    for r in rs.iter() {
        println!("{}", r);
    }
}
