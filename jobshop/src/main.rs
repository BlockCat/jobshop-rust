use disjunctgraph::{ Graph, LinkedGraph };
use jobshop::problem::*;
use std::fs::File;


fn main() {
    println!("Hello");

    let path = "bench_test.txt";

    let problem = Problem::read(path).unwrap();
    let graph = ProblemGraph::<LinkedGraph<ProblemNode>>::from(&problem).0;

    //println!("Problem statement: {:#?}", problem);
    //println!("Graph statement: {:?}", graph);    
    let graph = graph.into_directed().unwrap();
    println!("directed graph: {:?}", graph);
    graph.force_critical_path();
    let graph = graph.flip_edge(&5, &6).unwrap();
    graph.force_critical_path();

    let graph = graph.flip_edge(&2, &6).unwrap();
    graph.force_critical_path();

    let graph = graph.flip_edge(&3, &4).unwrap();
    graph.force_critical_path();

    //println!("cyclic graph: {:?}", graph.flip_edge(&3, &9).unwrap());

    /*let mut f = File::create("test.dot").unwrap();
    dot::render(&graph, &mut f).expect("Could not render to vizgraph");*/
}
