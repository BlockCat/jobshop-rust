extern crate jobshop;

use jobshop::problem::*;
use std::fs::File;


fn main() {
    println!("Hello");

    let path = "bench_la01.txt";

    let problem = Problem::read(path).unwrap();
    let graph = DisjunctiveGraph::from(&problem);

    //println!("Problem statement: {:#?}", problem);
    //println!("Graph statement: {:#?}", graph);

    let mut f = File::create("test.dot").unwrap();
    dot::render(&graph, &mut f).expect("Could not render to vizgraph");
}
