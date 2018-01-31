use parser::{VmCommand, Segment};

use std::io::Write;

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
            commands: Vec::new()
        }
    }
}

pub struct Graph {
    nodes: Vec<BasicBlock>,
}

impl Graph {
    pub fn build(commands: Vec<VmCommand>) -> Self {
        let block = BasicBlock::default(); 
        let mut graph = Graph { nodes: Vec::new() };
        let mut index = graph.add_node(block);
        for c in commands {
            match c {
                VmCommand::Goto(label) => {
                    if let Some(n) = graph.find_node_by_label(&label) {
                        graph.nodes[index].neighbors.push(n);
                    } else if graph.nodes[index].is_empty() {
                        graph.nodes[index].label = Some(label);
                    } else {
                        let dummy_block = BasicBlock { label: Some(label), ..Default::default() };
                        let new_index = graph.add_node(dummy_block);
                        graph.nodes[index].neighbors.push(new_index);
                    }
                    index = graph.add_node(BasicBlock::default());
                }
                VmCommand::Label(label) => {
                    if let Some(n) = graph.find_node_by_label(&label) {
                        graph.nodes[index].neighbors.push(n);
                        index = n;
                    } else if graph.nodes[index].is_empty() {
                        graph.nodes[index].label = Some(label);
                    } else {
                        let block = BasicBlock { label: Some(label), ..Default::default() };
                        let new_index = graph.add_node(block);
                        graph.nodes[index].neighbors.push(new_index);
                        index = new_index;
                    }
                }
                VmCommand::IfGoto(label) => {
                    if let Some(n) = graph.find_node_by_label(&label) {
                        graph.nodes[index].neighbors.push(n);
                    } else {
                        let block = BasicBlock { label: Some(label), ..Default::default() };
                        let new_index = graph.add_node(block);
                        graph.nodes[index].neighbors.push(new_index);
                    }
                    let not_taken_index = graph.add_node(BasicBlock::default());
                    graph.nodes[index].neighbors.push(not_taken_index);
                    index = not_taken_index;
                }
                cmd => graph.nodes[index].commands.push(cmd)
            }
        }
        graph
    }

    fn find_node_by_label(&self, label: &str) -> Option<usize> {
        self.nodes.iter().position(|n| n.label.is_some() && n.label.as_ref().unwrap() == label)
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
            let incoming = self.nodes.iter().any(|n| { 
                 if n.neighbors.iter().any(|&q| q == i) {
                     println!("{} -> {}", n.index, i);
                     true
                 } else {return false; }});
            if self.nodes[i].label.is_none() && self.nodes[i].commands.is_empty() && !incoming {
                println!("Remove {}", i);
                to_remove.push(i);
            }
        }
        for t in to_remove {
            self.nodes.remove(t);
        }
    }

    pub fn write_graphviz(&self, w: &mut Write) {
        w.write(b"digraph G {\n").unwrap();
        for i in 0..self.nodes.len() {
            let s: String = self.nodes[i].commands.iter().map(|s| format!("{:?}", s)).collect::<Vec<String>>().join("\\n");
            w.write_fmt(format_args!("{} [label=\"label={:?}\\n{}\"];\n", i, self.nodes[i].label, s)).unwrap();
            for n in self.nodes[i].neighbors.iter() {
                w.write_fmt(format_args!("{}->{};\n", i, n)).unwrap();
            }
        }
        w.write(b"}\n").unwrap();
    }
}