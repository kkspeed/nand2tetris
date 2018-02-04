extern crate decompiler;

use decompiler::parser::{vm_commands, VmCommand};
use decompiler::decompiler::Graph;
use decompiler::untyped_ir::UnTypedIR;
use std::convert::From;

use std::fs;
use std::io::{BufWriter};

fn main() {
    let input = fs::File::open("test.vm").unwrap();
    let output = fs::File::create("test.dot").unwrap();
    let mut writer = BufWriter::new(output);
    let mut buffer = Vec::new();
    let mut current_func = String::new();
    for c in vm_commands(input) {
        match c {
            VmCommand::FunDef(s, _)  => {
                if buffer.is_empty() {
                    current_func = s;
                    buffer = vec![];
                } else {
                    let new_buffer = buffer.drain(0..).collect();
                    let g: Graph<UnTypedIR> = From::from(Graph::build(new_buffer));
                    let rs = g.reconstruct_code();
                    println!("function {}(...) {{\n", current_func);
                    for r in rs.iter() {
                        println!("{};", r);
                    }
                    println!("}}\n");
                    current_func = s;
                }
            }
            x => {
                buffer.push(x);
            }
        }
    }

    let new_buffer = buffer.drain(0..).collect();
    let g: Graph<UnTypedIR> = From::from(Graph::build(new_buffer));
    let rs = g.reconstruct_code();
    println!("function {}(...) {{\n", current_func);
    for r in rs.iter() {
        println!("{};", r);
    }
    println!("}}\n");
    // let g: Graph<UnTypedIR> = From::from(Graph::build(vm_commands(input))); //.write_graphviz(&mut writer);
    // g.write_graphviz(&mut writer);
    // let rs = g.reconstruct_code();
    // for r in rs.iter() {
    //     println!("{}", r);
    // }
}
