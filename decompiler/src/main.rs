extern crate decompiler;

use decompiler::parser::vm_commands;
use decompiler::decompiler::Graph;

use std::fs;
use std::io::{BufWriter};

fn main() {
    let input = fs::File::open("test.vm").unwrap();
    let output = fs::File::create("test.dot").unwrap();
    let mut writer = BufWriter::new(output);
    Graph::build(vm_commands(input)).write_graphviz(&mut writer);
}
