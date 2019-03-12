use disjunctgraph::{ Graph, LinkedGraph };
use jobshop::problem::*;
use std::fs::File;


fn main() {
    println!("Hello");

    let path = "bench_test.txt";

    let problem = Problem::read(path).unwrap();
    let graph = ProblemGraph::<LinkedGraph<ProblemNode>>::from(&problem).0;

    println!("Problem statement: {:#?}", problem);
    println!("Graph statement: {:?}", graph);

    println!("directed graph: {:?}", graph.into_directed().unwrap());

    //println!("cyclic graph: {:?}", graph.flip_edge(&3, &5).unwrap());

    /*let mut f = File::create("test.dot").unwrap();
    dot::render(&graph, &mut f).expect("Could not render to vizgraph");*/
}
