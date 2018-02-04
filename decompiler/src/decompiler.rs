use parser::{VmCommand};
use std::fmt::Display;
use untyped_ir::*;

use std::io::Write;
use std::iter;
use std::collections::HashMap;

struct BasicBlock<CmdType> {
    index: usize,
    neighbors: Vec<usize>,
    label: Option<String>,
    commands: Vec<CmdType>,
}

impl<CmdType> BasicBlock<CmdType> {
    fn is_empty(&self) -> bool {
        self.neighbors.is_empty() && self.label.is_none() && self.commands.is_empty()
    }
}

impl<CmdType> Default for BasicBlock<CmdType> {
    fn default() -> Self {
        BasicBlock {
            index: 0,
            neighbors: Vec::new(),
            label: None,
            commands: Vec::new(),
        }
    }
}

pub struct Graph<CmdType> {
    nodes: Vec<BasicBlock<CmdType>>,
    edge_label: HashMap<(usize, usize), &'static str>,
}

impl From<Graph<VmCommand>> for Graph<UnTypedIR> {
    fn from(vm: Graph<VmCommand>) -> Self {
        let mut graph: Graph<UnTypedIR> = Graph { nodes: Vec::new(), edge_label: vm.edge_label.clone()};
        for i in 0..vm.nodes.len() {
            graph.nodes.push(BasicBlock {
                index: vm.nodes[i].index,
                neighbors: vm.nodes[i].neighbors.clone(),
                label: vm.nodes[i].label.clone(),
                commands: get_untyped_ir_from_vm_commands(&vm.nodes[i].commands),
            })
        }
        graph
    }
}

impl<CmdType: Display> Graph<CmdType> {
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

    fn add_node(&mut self, mut node: BasicBlock<CmdType>) -> usize {
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

    pub fn loop_headers(&self, dom: &Vec<Vec<usize>>) -> Vec<usize> {
        let mut result: Vec<usize> = iter::repeat(0).take(self.nodes.len()).collect();
        for i in 0..self.nodes.len() {
            for n in &self.nodes[i].neighbors {
                if dom[i][*n] == 1 {
                    result[*n] = 1;
                }
            }
        }
        result
    }
    
    pub fn dominate_nodes(&self) -> Vec<Vec<usize>> {
        let mut result = Vec::new();
        result.push(iter::repeat(0).take(self.nodes.len()).collect::<Vec<usize>>());
        result[0][0] = 1;
        for n in 1..self.nodes.len() {
           result.push(iter::repeat(1).take(self.nodes.len()).collect::<Vec<usize>>());
        }
        loop {
            let mut changed = false;
            for n in 1..self.nodes.len() {
                let mut t = iter::repeat(1).take(self.nodes.len()).collect::<Vec<usize>>();
                for k in self.pred(n) {
                    t = intersect(&result[k], &t);
                }
                t[n] = 1;
                for j in 0..t.len() {
                    if t[j] != result[n][j] {
                        result[n] = t;
                        changed = true;
                        break;
                    }
                }
            }
            if !changed {
                break;
            }
        }
        result
    }

    pub fn idom_nodes(&self, dom: &Vec<Vec<usize>>) -> Vec<usize> {
        let mut tmp : Vec<Vec<usize>> = iter::repeat(iter::repeat(0).take(self.nodes.len()).collect::<Vec<usize>>()).take(self.nodes.len()).collect();
        let mut idom = iter::repeat(0).take(self.nodes.len()).collect::<Vec<usize>>();

        for i in 0..self.nodes.len() {
            tmp[i] = dom[i].clone();
            tmp[i][i] = 0;
        }
        for i in 1..self.nodes.len() {
            for s in 0..self.nodes.len() {
                if tmp[i][s] == 1 {
                    for t in 0..self.nodes.len() {
                        if t != s && tmp[s][t] == 1 {
                            tmp[i][t] = 0;
                        }
                    }
                }
            }
        }

        for n in 1..self.nodes.len() {
            idom[n] = tmp[n].iter().position(|k| *k == 1).unwrap();
        }
        idom
    }

    pub fn write_graphviz(&self, w: &mut Write) {
        let d = self.dominate_nodes();
        let id = self.idom_nodes(&d);
        let lhs = self.loop_headers(&d);
        w.write(b"digraph G {\n").unwrap();
        for i in 0..self.nodes.len() {
            if self.nodes[i].neighbors.is_empty() && self.pred(i).is_empty() {
                continue;
            }
            let s = self.nodes[i].commands.iter()
                .map(|s| format!("{}", s))
                .collect::<Vec<String>>()
                .join("\\n");
            let mut doms = String::new();
            for dn in 0..d[i].len() {
                if d[i][dn] == 1 {
                    doms = format!("{} {}", doms, dn);
                }
            }
            w.write_fmt(format_args!(
                "{} [shape=box,label=\"{}\\n{}\\nlabel={}\\ndoms{{{}}}\\nidom={}\\n{}\"];\n",
                i,
                if lhs[i] == 1 { "header" } else { "" },
                i,
                self.nodes[i].label.as_ref().unwrap_or(&"".into()),
                doms,
                id[i],
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

impl Graph<VmCommand> {
    pub fn build(commands: Vec<VmCommand>) -> Graph<VmCommand> {
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
}

impl Graph<UnTypedIR> {
    pub fn reconstruct_code(&self) -> Vec<UnTypedIR> {
        let doms = self.dominate_nodes();
        let idoms = self.idom_nodes(&doms);
        let lhs = self.loop_headers(&doms);
        let (_, irs) = self.reconstruct_from_node_until(0, &doms, &idoms, &lhs, &|_| false);
        irs
    }

    fn reconstruct_from_node_until(&self, n: usize, doms: &Vec<Vec<usize>>, idoms: &Vec<usize>, lhs: &Vec<usize>, should_return: &Fn(usize) -> bool) -> (Option<usize>, Vec<UnTypedIR>) {
        if should_return(n) {
            return (Some(n), vec![]);
        }
        let mut result = self.nodes[n].commands.clone();
        if self.nodes[n].neighbors.is_empty() {
            return (None, result);
        }
        if lhs[n] == 1 {
            let t= self.taken_edge(n);
            let nt = self.not_taken_edge(n);
            let last_expr = result.pop().unwrap();
            let (_, loop_body) = self.reconstruct_from_node_until(nt, doms, idoms, lhs, &|i| i == n);
            let (_, continuation) = self.reconstruct_from_node_until(t, doms, idoms, lhs, should_return);
            let expr = UnTypedIR::While(Box::new(last_expr), loop_body, continuation);
            result.push(expr);
            return (None, result);
        }
        if self.nodes[n].neighbors.len() == 2 {
            let t = self.taken_edge(n);
            let nt = self.not_taken_edge(n);
            let last_expr = result.pop().unwrap();
            let mut else_body = vec![];
            let (n1, if_body) = self.reconstruct_from_node_until(t, doms, idoms, lhs, &|i| doms[i][t] == 0);
            let mut contn = nt;
            if self.pred(nt).len() == 1 {
                let (n2, eb) = self.reconstruct_from_node_until(nt, doms, idoms, lhs, &|i| doms[i][nt] == 0);
                else_body = eb;
                if n1.is_some() && n2.is_some() {
                    let contn1 = n1.unwrap();
                    let contn2 = n2.unwrap();

                    if contn1 != contn2 {
                        panic!("If statements not reach same node if {} vs else {}", contn1, contn2);
                    }
                    contn = contn2;
                } 
            }
            let (r, rs) = self.reconstruct_from_node_until(contn, doms, idoms, lhs, should_return);
            let expr = UnTypedIR::If(Box::new(last_expr), if_body, else_body, rs);
            result.push(expr);
            return (r, result);
        }
        let mut ret = None;
        for i in self.nodes[n].neighbors.iter() {
            let (r, rs) = self.reconstruct_from_node_until(*i, doms, idoms, lhs, should_return);
            ret = r;
            result.extend(rs);
        }
        (ret, result)
    }

    fn taken_edge(&self, n: usize) -> usize {
        for i in self.nodes[n].neighbors.iter() {
            if self.edge_label.get(&(n, *i)).is_some() && *self.edge_label.get(&(n, *i)).unwrap() == "if-goto" {
                return *i;
            }
        }
        panic!("cannot find taken edge");
    }

    fn not_taken_edge(&self, n: usize) -> usize {
        for i in self.nodes[n].neighbors.iter() {
            if self.edge_label.get(&(n, *i)).is_none() || *self.edge_label.get(&(n, *i)).unwrap() != "if-goto" {
                return *i;
            }
        }
        panic!("cannot find not taken edge");
    }
}

fn intersect(v1: &[usize], v2: &[usize]) -> Vec<usize> {
    let mut result = Vec::new();
    for i in 0..v1.len() {
        result.push(v1[i] * v2[i]);
    }
    result
}