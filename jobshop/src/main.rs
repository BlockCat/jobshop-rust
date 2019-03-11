
use jobshop::problem::*;
use std::fs::File;

enum Relation {
    Successor,
    Predecessor,
    Disjunctive,
    Unknown
}

struct RelationShip {
    relation: Relation,
    prev: u16,
    next: u16
}

fn main() {
    println!("Hello");


    println!("Container: {}", std::mem::size_of::<Relation>());
    println!("Success: {}", std::mem::size_of::<RelationShip>());
    println!("i32: {}, u32: {}", std::mem::size_of::<i32>(), std::mem::size_of::<u32>());

    let path = "bench_test.txt";

    let problem = Problem::read(path).unwrap();
    let graph = DisjunctiveGraph::from(&problem);

    //println!("Problem statement: {:#?}", problem);
    //println!("Graph statement: {:#?}", graph);

    /*let mut f = File::create("test.dot").unwrap();
    dot::render(&graph, &mut f).expect("Could not render to vizgraph");*/
}
