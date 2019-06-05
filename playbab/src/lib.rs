#![feature(drain_filter)]

extern crate disjunctgraph;

mod node;
mod task_interval;
mod propagation;

use std::collections::VecDeque;

use disjunctgraph::{ GraphNode, ConstrainedNode, NodeId, Graph };
use itertools::Itertools;

// Constrained graph ;
pub type CGraph = disjunctgraph::LinkedGraph<node::Node>;

type TaskInterval<'a> = task_interval::TaskInterval<'a, CGraph>;

const PAR: u32 = 3;

// What is still needed?
// There is no propagation of constraints,
// Operations that have no disjunctions left are looked at too.
// I believe cycles can occur? ()
pub fn branch_and_bound(mut root: CGraph, resources: usize, max_makespan: u32) -> CGraph {
    root.init_weights();//.expect("Problem with makespan is not feasible");    
    dbg!(crate::propagation::search_orders(&mut root, max_makespan).unwrap());

    let resources = (1..=resources).collect::<Vec<_>>();
    for resource in &resources {
        crate::propagation::edge_finding(*resource as u32, &mut root, max_makespan).unwrap();
    }
    //root.init_weights();
    //root.search_orders(max_makespan);
    root.init_weights();
    
    //println!("{:?}", root);
    let mut upper_bound = max_makespan;
    let mut current_best = root.clone();

    let mut stack: VecDeque<CGraph> = VecDeque::new();
    stack.push_front(root);

    
    let mut node_evaluations = 0;
    while let Some(node) = stack.pop_front() {        
        node_evaluations += 1;

        // Check if graph has disjunctions left.
        if !node.has_disjunctions() {
            // We are a complete schedule!
            
            let length = dbg!(node.critical_length().expect("Could not calculate critical length"));
            let lb = lower_bound(&node, upper_bound, &resources);
            if length <= upper_bound {
                upper_bound = length;
                current_best = node;
            }

            println!("We got one of length: {} or is it {}?", length, lb);
        } else {
            if lower_bound(&node, upper_bound, &resources) > upper_bound {                
                continue;
            }
            //println!("Disjunctions left: {}", node.total_disjunctions());
            if let Ok(pairs) = next_pair(&resources, &node, upper_bound) {
                for (free_node, (head, tail)) in pairs {
                    if head + free_node.weight() + tail > upper_bound {
                        continue;
                    }
                    
                    let mut graph = node.clone();

                    println!("Total slack before: {}", graph.nodes().iter().map(|n| {
                        upper_bound - n.head() - n.weight() - n.tail()
                    }).sum::<u32>());

                    graph[free_node.id()].set_tail(tail);
                    graph[free_node.id()].set_head(head);
                    
                    let r1 = propagation::propagate_head(free_node, &mut graph, upper_bound)
                        .and_then(|_| propagation::propagate_tail(free_node, &mut graph, upper_bound))
                        .and_then(|_| {
                            while propagation::search_orders(&mut graph, upper_bound)? {}
                            Ok(())
                        });
                        
                    
                    match r1 {
                        Ok(_) => {

                            println!("Total slack after: {}", graph.nodes().iter().map(|n| {
                                upper_bound - n.head() - n.weight() - n.tail()
                            }).sum::<u32>());
                            if lower_bound(&graph, max_makespan, &resources) <= upper_bound {
                                stack.push_front(graph);                            
                            }
                        },
                        Err(e1) => println!("Error: {:?}", e1)
                    }
                }
            }
        }
    }
    println!("Node evaluations: {}", node_evaluations);
    current_best
}



fn next_pair<'a>(resources: &[usize], graph: &'a CGraph, upper_bound: u32) -> Result<Vec<(&'a node::Node, (u32, u32))>, String> {
    // Naive shit to do it.
    // We take the resource with most freedom and many successors/predecessors.
    // freedom * (successors + predecessors)
    let free_node = graph.nodes().iter().max_by_key(|n| {
        let freedom = upper_bound - n.tail() - n.weight() - n.head();
        let successors = graph.successors(&n.id())
            .filter(|o| n.head() + (freedom >> 1) > o.head()) // if my head changes, successors head changes
            .count() as u32;
        let predecessors = graph.predecessors(&n.id())
            .filter(|o| n.tail() + (freedom >> 1) > o.tail()) // if my head changes, successors head changes
            .count() as u32;
        //let disjunctions = graph.disjunctions(&n.id()).count() as u32;
        freedom * (successors + predecessors)// * disjunctions
    }).unwrap();

    let head = free_node.head();
    let tail = free_node.tail();

    let freedom = (upper_bound - free_node.tail()) - free_node.weight() - free_node.head();
    
    let new_tail = tail + freedom >> 1;
    let new_head = head + freedom >> 1;

    // We want to split in two for now,
    // [est, (est + lct) / 2]
    // [(est + lct) / 2, lct]

    // [head, (est + ub - tail) / 2]
    // 

    Ok(vec!((free_node, (head, new_tail + free_node.weight())), (free_node, (new_head, tail))))
}


/// According to: Adjustment of heads and tails for the job-shop problem (J. Carlier and E. Pinson)
/// Chapter 4.4: Lower bound
/// Warning: Does not implement all three bounds.
/// A study of lower bounds is acceptable
fn lower_bound(graph: &CGraph, upper_bound: u32, resources: &[usize]) -> u32 {    
    let resources = resources.iter()
        .map(|resource| graph.nodes().iter().filter(move |n| n.machine_id() == Some(*resource as u32))) // Returns an I_k on machine M_k
        .collect::<Vec<_>>();
        
    let d2 = graph.nodes().iter().map(|n| n.head() + n.weight() + n.tail()).max().unwrap();
    /*let d1 = resources.into_iter()
        .flat_map(|resource| resource.combinations(2)) // Take all combinations of 2
        .map(|comb| {
            let n1 = &comb[0];
            let n2 = &comb[1];

            let a = n1.head() + n1.weight() + n2.weight() + n2.tail();
            let b = n2.head() + n2.weight() + n1.weight() + n2.tail();
            std::cmp::max(a, b)
        })        
        .filter(|d| d <= &upper_bound)
        .max().unwrap_or(d2 + 1);*/

    //std::cmp::max(d1 - 1, d2)
    d2
}
