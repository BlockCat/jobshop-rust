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
    root.search_orders(max_makespan);
    for resource in 1..=resources {
        crate::propagation::edge_finding(resource as u32, &mut root, max_makespan).unwrap();
    }
    root.init_weights();
    
    
    let mut upper_bound = max_makespan;
    let mut current_best = root.clone();

    let mut stack: VecDeque<CGraph> = VecDeque::new();
    stack.push_front(root);

    let resources = (1..=resources).collect::<Vec<_>>();

    while !stack.is_empty() {
        let node = stack.pop_front().unwrap();        

        // Check if graph has disjunctions left.
        if !node.has_disjunctions() {
            // We are a complete schedule!
            
            let length = dbg!(node.critical_length().expect("Could not calculate critical length"));
            if length <= upper_bound {
                upper_bound = length;
                current_best = node;
            }
            println!("We got one of length");
        } else {
            if node.nodes().iter().any(|n| upper_bound < n.head() + n.weight() + n.tail()) {
                continue;
            }
            for (t1, t2) in next_pair(&resources, &node, upper_bound) {
                debug_assert!(t1.head() + t1.weight() + t2.weight() + t2.tail() <= upper_bound);
                let mut graph = node.clone();
                graph.fix_disjunction(t1, t2).expect("Could not fix disjunction");
                //println!("g: {}", graph);
                if propagation::propagate_fixation(&mut graph, t1, t2, upper_bound).is_ok() {                    
                    if dbg!(lower_bound(&graph, max_makespan, &resources)) <= upper_bound {
                        stack.push_front(graph);
                    }
                }
            }
        }
    }

    current_best
}



fn next_pair<'a>(resources: &[usize], graph: &'a CGraph, upper_bound: u32) -> Vec<(&'a node::Node, &'a node::Node)> {

    // Calculate the critical task interval for each resource/machine
    // Returns true if machine still has operations that need to be ordered
    let resource_filter = |id: &&usize| -> bool {
        graph.nodes().iter()
            .filter(|x| x.machine_id() == Some(**id as u32))
            .any(|x| graph.node_has_disjunction(x))
    };
    let criticals: Vec<(usize, TaskInterval)> = resources.iter()
        .filter(resource_filter)
        .filter_map(|id| crit(*id, graph, upper_bound).map(|x| (*id, x)))
        .collect();
    
    // Find the resource with the most constrained task interval
    let (resource_id, crit) = criticals.into_iter()
        .min_by_key(|(id, cr)| {
            let resource_slack = resource_slack(*id as u32, graph, upper_bound);
            cr.slack() as u32 * resource_slack * std::cmp::min(PAR, num_choices(cr) as u32)
        })
        .expect("Can't find critical");
    
    debug_assert!(!crit.nodes.iter().all(|x| !graph.node_has_disjunction(*x)));

    let t1 = crit.nodes.iter().min_by_key(|x| x.est()).expect("Could not extract left bound node"); // Get left bounded node on the task interval
    let t2 = crit.nodes.iter().max_by_key(|x| x.lct(upper_bound)).expect("Could not extract right bound node"); // Get right bounded node on the task interval
    
    let resource_nodes = graph.nodes().iter().filter(|x| x.machine_id() == Some(resource_id as u32)).collect_vec();
    let crit_slack = crit.slack();

    let can_be_first = |t: &&&node::Node| -> bool {
        t.est() <= t1.est() + crit_slack 
        && t.id() != t1.id()
        && graph.has_disjunction(&t1.id(), &t.id())        
    };
    let can_be_last = |t: &&&node::Node| -> bool {
        t.lct(upper_bound) + crit_slack >= t2.lct(upper_bound)
        && t.id() != t2.id()
        && graph.has_disjunction(&t.id(), &t2.id())        
    };
    
    let s1 = resource_nodes.iter()
        .filter(can_be_first)
        .collect_vec();
    let s2 = resource_nodes.iter()
        .filter(can_be_last)
        .collect_vec();

    debug_assert!(s1.len() > 0 || s2.len() > 0);

    if (s1.len() <= s2.len() && s1.len() > 0) || s2.len() == 0 {
        let delta = crit.nodes.iter()
            .filter(|x| x.id() != t1.id())
            .map(|x| x.est()).min().expect("No min S1 found") - t1.est();

        let t = s1.iter()
            .min_by_key(|t| left_bounded_entropy(t1, t, upper_bound, &crit, delta))
            .expect("Could not minimize h1");

        if g(t1, t, upper_bound) <= g(t, t1, upper_bound) {
            vec!((t1, t), (t, t1))
        } else {
            vec!((t, t1), (t1, t))
        }
    } else {
        let delta = t2.lct(upper_bound) - crit.nodes.iter()
            .filter(|x| x.id() != t2.id())
            .map(|x| x.lct(upper_bound)).max().expect("No max S2 found");
        let t = s2.iter()
            .min_by_key(|t| right_bounded_entropy(t, t2, upper_bound, &crit, delta))
            .expect("Could not minimize h2");

        if g(t, t2, upper_bound) <= g(t2, t, upper_bound) {
            vec!((t, t2), (t2, t))
        } else {
            vec!((t2, t), (t, t2))
        }
    }
}


/// Find the critical on a resource, if there is no found then there inconsistency
/// It can happen that a resource is already completely scheduled.
fn crit<'a>(resource_id: usize, graph: &'a CGraph, upper_bound: u32) -> Option<TaskInterval<'a>> {
    
    // Get the nodes on the resources
    
    let task_intervals = task_interval::find_task_intervals(resource_id as u32, graph, upper_bound);
    
    debug_assert!(task_intervals.len() > 0);

    // Only resources are considered that have more than 1 node anyway.
    task_intervals.into_iter()        
        .min_by_key(|x| x.slack() as u32 * num_choices(x) as u32)
}

/// Calculate the slack for all operations on a resource
fn resource_slack(resource: u32, graph: &CGraph, upper_bound: u32) -> u32 {
    let (min, max, p) = graph.nodes().iter()
        .filter(|x| x.machine_id() == Some(resource as u32))
        .fold((std::u32::MAX, 0, 0), |(min, max, p), x| {
            let min = std::cmp::min(min, x.est());
            let max = std::cmp::max(max, x.lct(upper_bound));
            let p = p + x.weight();
            (min, max, p)
        });
    max - min - p
}


/// When fixing t1 -> t2, this function returns the expected reduction in entropy on the domain
fn left_bounded_entropy(t1: &node::Node, tb: &node::Node, max_makespan: u32, task_interval: &TaskInterval, delta: u32) -> u32 {
    let t1_tb = g(t1, tb, max_makespan);
    let tb_t1 = g(tb, t1, max_makespan);

    // If this assert fails then that means that t2 cannot be placed to the left
    debug_assert!(task_interval.upper() >= tb.est() + task_interval.processing);

    let new_slack = task_interval.upper() - tb.est() - task_interval.processing;
    let fff = evaluation(new_slack, delta, max_makespan);

    std::cmp::max(t1_tb, std::cmp::min(tb_t1, fff))
}

fn right_bounded_entropy(ta: &node::Node, t2: &node::Node, upper_bound: u32, task_interval: &TaskInterval, delta: u32) -> u32 {
    let ta_t2 = g(ta, t2, upper_bound);
    let t2_ta = g(t2, ta, upper_bound);

    // If this assert fails then that means that t2 cannot be placed
    debug_assert!(ta.lct(upper_bound) >= task_interval.lower() + task_interval.processing);

    let new_slack = ta.lct(upper_bound) - task_interval.lower() - task_interval.processing;
    let fff = evaluation(new_slack, delta, upper_bound);

    std::cmp::max(ta_t2, std::cmp::min(t2_ta, fff))
}

/// Assess impact of an ordering ta -> tb
fn g(ta: &node::Node, tb: &node::Node, upper_bound: u32) -> u32 {
    let da = ta.lct(upper_bound).saturating_sub(tb.lst(upper_bound));
    let db = (ta.head() + ta.weight()).saturating_sub(tb.head());    

    let a = evaluation(upper_bound - ta.tail() - ta.head() - ta.weight(), da, upper_bound);
    let b = evaluation(upper_bound - tb.tail() - tb.head() - tb.weight(), db, upper_bound);

    std::cmp::min(a, b)
}

fn evaluation(slack: u32, delta: u32, max_makespan: u32) -> u32 {
    if delta == 0 {
        max_makespan
    } else if slack < delta {
        0
    } else {
        (slack - delta).pow(2) / slack
    }
}


fn num_choices(task_interval: &TaskInterval) -> usize {
    std::cmp::min(task_interval.nc_start.len(),  task_interval.nc_end.len())
}

/// According to: Adjustment of heads and tails for the job-shop problem (J. Carlier and E. Pinson)
/// Chapter 4.4: Lower bound
/// Warning: Does not implement all three bounds.
/// A study of lower bounds is acceptable
fn lower_bound(graph: &CGraph, upper_bound: u32, resources: &[usize]) -> u32 {    
    let resources = resources.iter()
        .map(|resource| graph.nodes().iter().filter(move |n| n.machine_id() == Some(*resource as u32))) // Returns an I_k on machine M_k
        .collect::<Vec<_>>();
        
    let d1 = resources.into_iter()
        .flat_map(|resource| resource.combinations(2)) // Take all combinations of 2
        .map(|comb| {
            let n1 = &comb[0];
            let n2 = &comb[1];

            let a = n1.head() + n1.weight() + n2.weight() + n2.tail();
            let b = n2.head() + n2.weight() + n1.weight() + n2.tail();
            std::cmp::max(a, b)
        })        
        .filter(|d| d <= &upper_bound)
        .max().unwrap_or(upper_bound + 1);
    d1 - 1
}
