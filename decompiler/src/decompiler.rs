use parser::{Segment, VmCommand};
use untyped_ir::*;

use std::io::Write;
use std::collections::HashMap;

struct BasicBlock {
    index: usize,
    neighbors: Vec<usize>,
    label: Option<String>,
    commands: Vec<VmCommand>,
}

impl BasicBlock {
    fn is_empty(&self) -> bool {
        self.neighbors.is_empty() && self.label.is_none() && self.commands.is_empty()
    }
}

impl Default for BasicBlock {
    fn default() -> Self {
        BasicBlock {
            index: 0,
            neighbors: Vec::new(),
            label: None,
            commands: Vec::new(),
        }
    }
}

pub struct Graph {
    nodes: Vec<BasicBlock>,
    edge_label: HashMap<(usize, usize), &'static str>,
}

impl Graph {
    pub fn build(commands: Vec<VmCommand>) -> Self {
        let block = BasicBlock::default();
        let mut graph = Graph {
            nodes: Vec::new(),
            edge_label: HashMap::new(),
        };
        let mut index = graph.add_node(block);
        for c in commands {
            match c {
                VmCommand::Goto(label) => {
                    if let Some(n) = graph.find_node_by_label(&label) {
                        graph.nodes[index].neighbors.push(n);
                        graph.edge_label.insert((index, n), "goto");
                    } else if graph.nodes[index].is_empty() {
                        graph.nodes[index].label = Some(label);
                    } else {
                        let dummy_block = BasicBlock {
                            label: Some(label),
                            ..Default::default()
                        };
                        let new_index = graph.add_node(dummy_block);
                        graph.nodes[index].neighbors.push(new_index);
                        graph.edge_label.insert((index, new_index), "goto");
                    }
                    index = graph.add_node(BasicBlock::default());
                }
                VmCommand::Label(label) => if let Some(n) = graph.find_node_by_label(&label) {
                    graph.nodes[index].neighbors.push(n);
                    index = n;
                } else if graph.nodes[index].is_empty() {
                    graph.nodes[index].label = Some(label);
                } else {
                    let block = BasicBlock {
                        label: Some(label),
                        ..Default::default()
                    };
                    let new_index = graph.add_node(block);
                    graph.nodes[index].neighbors.push(new_index);
                    index = new_index;
                },
                VmCommand::IfGoto(label) => {
                    if let Some(n) = graph.find_node_by_label(&label) {
                        graph.nodes[index].neighbors.push(n);
                        graph.edge_label.insert((index, n), "if-goto");
                    } else {
                        let block = BasicBlock {
                            label: Some(label),
                            ..Default::default()
                        };
                        let new_index = graph.add_node(block);
                        graph.nodes[index].neighbors.push(new_index);
                        graph.edge_label.insert((index, new_index), "if-goto");
                    }
                    let not_taken_index = graph.add_node(BasicBlock::default());
                    graph.nodes[index].neighbors.push(not_taken_index);
                    index = not_taken_index;
                }
                cmd => graph.nodes[index].commands.push(cmd),
            }
        }
        graph.shrink();
        graph
    }

    fn find_node_by_label(&self, label: &str) -> Option<usize> {
        self.nodes
            .iter()
            .position(|n| n.label.is_some() && n.label.as_ref().unwrap() == label)
    }

    fn pred(&self, node: usize) -> Vec<usize> {
        self.nodes
            .iter()
            .enumerate()
            .filter_map(|(index, n)| if n.neighbors.iter().any(|&i| i == node) {
                Some(index)
            } else {
                None
            })
            .collect()
    }

    fn add_node(&mut self, mut node: BasicBlock) -> usize {
        let index = self.nodes.len();
        node.index = index;
        self.nodes.push(node);
        return index;
    }

    fn add_edge(&mut self, s: usize, d: usize) {
        if self.nodes[s].neighbors.iter().find(|&n| *n == d).is_none() {
            self.nodes[s].neighbors.push(d)
        }
    }

    fn shrink(&mut self) {
        let mut to_remove = Vec::new();
        for i in 0..self.nodes.len() {
            let incoming = !self.pred(i).is_empty();
            if self.nodes[i].label.is_none() && self.nodes[i].commands.is_empty() && !incoming {
                to_remove.push(i);
            }
        }
        for t in to_remove {
            self.nodes[t].neighbors.clear();
        }
    }

    pub fn write_graphviz(&self, w: &mut Write) {
        w.write(b"digraph G {\n").unwrap();
        for i in 0..self.nodes.len() {
            let irs = get_untyped_ir_from_vm_commands(&self.nodes[i].commands);
            let s = irs.iter()
                .map(|s| format!("{}", s))
                .collect::<Vec<String>>()
                .join("\\n");
            w.write_fmt(format_args!(
                "{} [shape=box,label=\"label={}\\n{}\"];\n",
                i,
                self.nodes[i].label.as_ref().unwrap_or(&"".into()),
                s
            )).unwrap();
            for n in self.nodes[i].neighbors.iter() {
                w.write_fmt(format_args!(
                    "{}->{}[label=\"{}\"];\n",
                    i,
                    n,
                    self.edge_label.get(&(i, *n)).unwrap_or(&"")
                )).unwrap();
            }
        }
        w.write(b"}\n").unwrap();
    }
}
