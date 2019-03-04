extern crate jobshop;

use jobshop::problem::*;

fn main() {
    println!("Hello");

    let path = "bench_la01.txt";

    let problem = Problem::read(path);

    println!("Problem statement: {:#?}", problem.unwrap());
}
