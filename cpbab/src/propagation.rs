//! Propagation in the graph: The main idea is to use intervals
//! Up(S) - low(S) >= d(S) has to satisfy every different Taskinterval.
//! 
//! The paper describes three different constrraint rules:
//! Symmetrical rules apply to upper bounds
//! 1. Ordering rules: 
//! For every (t1, t2) (t1 -> t2 ^ low(t2) < low(t1) + d(t1)) => low(t2) = low(t1) + d(t1)
//! 
//! 2. Edge finding:
//! Determine if a task can be the first or the last in a given task interval:
//! for a task t in S, if up(S) - low(t) - d(S) < 0 then t cannot be first in S
//! for every (t, S) (up(S) - low(t) - d(S) < 0) => low(t) >= min low(t_i + d(t_i); t_i in S - {t})
//! 
//! 3. Exclusion
//! Tries to order tasks and intervals.
//! for a t not in S, if up(S) - low(t) - d(S) - d(t) < 0 
//! then t must be executed after some tasks in S
//! 
//! two special cases exist that say that t has to be executed after all tasks in S
//! packed: up(S) - low(S) < d(S) + d(t)
//! is_after: low(t) + d(t) > max(up(t_i) + d(t_i); t_i in S)
//! 
//! rules used:
//! for all (t, S) (up(S) - low(t) - d(S) - d(t) < 0) =>
//! if packed or is_after then low(t) >= low(S) + d(S) and for all t_i in S up(t_i) <= up(t) - d(t)
//! otherwise low(t) >= min(t_i + d(t_i); t_i in S)
//! 
//! Refinements are described for exclusion and edge-finding
//! 
//! Ordering of rules triggered:
//! 1. ordering rules triggered upon changed to up(t), low(t) or a fixation of a disjunction
//! 2. edge-finding rules when 
//! 
use disjunctgraph::{ Graph, ConstrainedNode, GraphNode, NodeId };
use itertools::Itertools;
use std::collections::HashSet;

use crate::task_interval;


pub fn propagate_tail<I: Graph>(node: &impl NodeId, graph: &mut I, upper_bound: u32) -> Result<HashSet<usize>, ()> where I::Node: ConstrainedNode {
    use std::collections::VecDeque;

    let mut stack: VecDeque<(usize, u32)> = VecDeque::new();
    let mut changed: HashSet<usize> = HashSet::new();
    
    { 
        // sup my tail changed, so yours might too
        // so yours has to be at least my tail + my weight
        let next_tail = graph[node.id()].tail() + graph[node.id()].weight();
        stack.extend(graph.predecessors(node).map(|p| (p.id(), next_tail)));
    }

    while let Some((id, min_tail)) = stack.pop_back() {
        let node = &mut graph[id];
    
        if min_tail > node.tail() {
            if node.head() + node.weight() + min_tail <= upper_bound {
                node.set_tail(min_tail);
                let next_tail = min_tail + node.weight();
                stack.extend(graph.predecessors(&id).map(|p| (p.id(), next_tail)));
                changed.insert(id);
            } else {
                return Err(());
            }
        }
    }

    Ok(changed)
}


pub fn propagate_head<I: Graph>(node: &impl NodeId, graph: &mut I, upper_bound: u32) -> Result<HashSet<usize>, ()> where I::Node: ConstrainedNode {
    use std::collections::VecDeque;

    let mut stack: VecDeque<(usize, u32)> = VecDeque::new();
    let mut changed: HashSet<usize> = HashSet::new();
    
    {
        let next_head = graph[node.id()].head() + graph[node.id()].weight();
        stack.extend(graph.successors(node).map(|s| (s.id(), next_head)));
    }

    while let Some((id, min_head)) = stack.pop_back() {
        let node = &mut graph[id];
    
        if min_head > node.head() {
            if min_head + node.weight() + node.tail() <= upper_bound {
                node.set_head(min_head);
                let next_head = min_head + node.weight();
                stack.extend(graph.successors(&id).map(|s| (s.id(), next_head)));
                changed.insert(id);
            } else {
                return Err(());
            }
        }
    }

    Ok(changed)
}

pub fn edge_finding<I: Graph + std::fmt::Debug>(resource: u32, graph: &mut I, upper_bound: u32) -> Result<(), ()> where I::Node: ConstrainedNode + std::fmt::Debug {

    let tis = task_interval::find_task_intervals(resource, graph, upper_bound);

    let starts: Vec<(usize, usize)> = tis.iter()
        .filter(|ti| ti.nc_start.len() == 1)
        .flat_map(|ti| {
            let id = ti.nc_start[0].id();
            ti.nodes.iter()
                .filter(|n| graph.has_disjunction(&n.id(), &id))
                .map(|n| (id, n.id()))
                .collect_vec()
        }).collect();

    let ends: Vec<(usize, usize)> = tis.iter()
        .filter(|ti| ti.nc_end.len() == 1)
        .flat_map(|ti| {
            let id = ti.nc_end[0].id();
            ti.nodes.iter()
                .filter(|n| graph.has_disjunction(&n.id(), &id))
                .map(|n| (n.id(), id))
                .collect_vec()
        }).collect();

    for (other, end) in ends {
        graph.fix_disjunction(&other, &end).or(Err(())).or(Err(()))?;
        propagate_head(&end, graph, upper_bound)?;
        propagate_tail(&other, graph, upper_bound)?;
    }
    
    for (start, other) in starts {
        graph.fix_disjunction(&start, &other).or(Err(())).or(Err(()))?;
        propagate_head(&other, graph, upper_bound)?;
        propagate_tail(&start, graph, upper_bound)?;
    }

    Ok(())
}
/// Propagate a fixation node_1 -> node_2
pub fn propagate_fixation<I: Graph + std::fmt::Debug>(graph: &mut I, node_1: &impl NodeId, node_2: &impl NodeId, upper_bound: u32) -> Result<(), ()> where I::Node: ConstrainedNode + std::fmt::Debug {
    
    propagate_head(node_1, graph, upper_bound)?;
    propagate_tail(node_2, graph, upper_bound)?;
    
    graph.search_orders(upper_bound);

    let resource = graph[node_1.id()].machine_id().unwrap();

    edge_finding(resource, graph, upper_bound)?;
    graph.search_orders(upper_bound);
    

    Ok(())
}


// What do we need to do: 
// The paper tells us first about propagation,
// then interval maintenance
// these two are interleaved, maintenance triggers propagation
// and propagation triggers maintenance
