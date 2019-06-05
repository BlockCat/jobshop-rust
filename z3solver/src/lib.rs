#![feature(drain_filter)]

extern crate disjunctgraph;
extern crate z3;


use z3::*;
use disjunctgraph::{ LinkedGraph, NodeId, Graph, GraphNode };


mod node;

pub type LGraph = LinkedGraph<node::Node>;


pub fn solve(problem: LGraph) -> Vec<u32>  {
    let config = Config::new();
    let context = Context::new(&config);
    let optimizer = Optimize::new(&context);  

    // We have to assign starting times to nodes,
    // So create starting times
    let starting_times: Vec<Ast> = problem.nodes()
        .iter()
        .map(|node| context.numbered_int_const(node.id() as u32))
        .collect();

    let processing_times: Vec<Ast> = problem.nodes()
        .iter()
        .map(|node| context.from_u64(node.weight() as u64))
        .collect();

    let zero = context.from_u64(0);
    
    for node in problem.nodes() {
        let si = &starting_times[node.id()];
        let pi = &processing_times[node.id()];

        let sipi = &si.add(&[&pi]);
        // For each node add directed edges
        // s_i + p_i <= s_j
        for successor in problem.successors(node) {
            optimizer.assert(&sipi.le(&starting_times[successor.id()]));
        }       

        optimizer.assert(&si.ge(&zero));

        // For each node add disjunctions
        // s_i + p_i <= s_j or s_j + p_j <= s_i
        for disjunction in problem.disjunctions(node).filter(|other| other.id() > node.id()) {
            let sj = &starting_times[disjunction.id()];
            let pj = &processing_times[disjunction.id()];

            let sipisj = &si.add(&[&pi]).le(&sj); // si + pi <= sj
            let sjpjsi = &sj.add(&[&pj]).le(&si); // sj + pj <= si

            optimizer.assert(&sipisj.or(&[&sjpjsi]));
        }
    }
    
    // We want to minimize the latest ending time.
    optimizer.minimize(&starting_times.last().unwrap());

    if optimizer.check() {
        let model = optimizer.get_model();
        starting_times
            .iter()
            .map(|st| model.eval(st).expect("not evaluated").as_u64().unwrap() as u32)
            .collect()
    } else {
        panic!("Could not solve")
    }    
}