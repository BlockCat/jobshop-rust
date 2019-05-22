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

use crate::task_interval::TaskInterval;
use crate::task_interval;


pub fn propagate_lct<I: Graph>(node: &impl NodeId, graph: &mut I) -> Result<HashSet<usize>, ()> where I::Node: ConstrainedNode {
    use std::collections::VecDeque;

    let mut stack: VecDeque<(usize, u32)> = VecDeque::new();
    let mut changed: HashSet<usize> = HashSet::new();
    
    { // sup my lct changed, so yours might too
        let next_lct = graph[node.id()].lst();
        stack.extend(graph.predecessors(node).map(|p| (p.id(), next_lct)));
    }

    while let Some((id, max_lct)) = stack.pop_back() {
        let node = &mut graph[id];
    
        if node.lct() > max_lct {
            if node.feasible_lct(max_lct) {
                node.set_lct(max_lct);
                let next_lct = node.lst();
                stack.extend(graph.predecessors(&id).map(|p| (p.id(), next_lct)));
                changed.insert(id);
            } else {
                return Err(());
            }
        }
    }

    Ok(changed)
}


pub fn propagate_est<I: Graph>(node: &impl NodeId, graph: &mut I) -> Result<HashSet<usize>, ()> where I::Node: ConstrainedNode {
    use std::collections::VecDeque;

    let mut stack: VecDeque<(usize, u32)> = VecDeque::new();
    let mut changed: HashSet<usize> = HashSet::new();
    
    {
        let next_est = graph[node.id()].est() + graph[node.id()].weight();
        stack.extend(graph.successors(node).map(|s| (s.id(), next_est)));
    }

    while let Some((id, min_est)) = stack.pop_back() {
        let node = &mut graph[id];
    
        if node.est() < min_est {
            if node.feasible_est(min_est) {
                node.set_est(min_est);
                let next_est = min_est + node.weight();

                stack.extend(graph.successors(&id).map(|s| (s.id(), next_est)));
                changed.insert(id);
            } else {
                return Err(());
            }
        }
    }

    Ok(changed)
}

pub fn edge_finding<I: Graph + std::fmt::Debug>(resource: u32, graph: &mut I) -> Result<(), ()> where I::Node: ConstrainedNode + std::fmt::Debug {

    let tis = task_interval::find_task_intervals(resource, graph);

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


    println!("Found: {:?}", starts);
    println!("Found: {:?}", ends);

    for (other, end) in ends {
        graph.fix_disjunction(&other, &end).or(Err(())).or(Err(()))?;
        propagate_est(&other, graph)?;
        propagate_est(&end, graph)?;
    }
    
    for (start, other) in starts {
        graph.fix_disjunction(&start, &other).or(Err(())).or(Err(()))?;
        propagate_est(&start, graph)?;
        propagate_est(&other, graph)?;
    }

    Ok(())
}
/// Propagate a fixation node_1 -> node_2
pub fn propagate_fixation<I: Graph + std::fmt::Debug>(graph: &mut I, node_1: &impl NodeId, node_2: &impl NodeId) -> Result<(), ()> where I::Node: ConstrainedNode + std::fmt::Debug {
    
    propagate_est(node_1, graph)?;
    propagate_lct(node_2, graph)?;
    graph.search_orders();

    let resource = graph[node_1.id()].machine_id().unwrap();

    edge_finding(resource, graph)?;
    graph.search_orders();
    
    Ok(())
}


// What do we need to do: 
// The paper tells us first about propagation,
// then interval maintenance
// these two are interleaved, maintenance triggers propagation
// and propagation triggers maintenance

// They say for maintenance:
// 1. Do actions that do not have to be propagated (negative shit)
// 2.

struct PropagatedTaskInterval {
    lower: usize,
    upper: usize,
    processing: u32,
    nodes: Vec<usize>
}

fn propagate_node<'a, T: Graph>(node: &impl NodeId, graph: &mut T) where T::Node: ConstrainedNode + std::fmt::Debug {    
    // Need to be maintained
    let interval_to_ref = |x: TaskInterval<T>| -> PropagatedTaskInterval {
        PropagatedTaskInterval { 
            lower: x.lower.id(), 
            upper: x.upper.id(), 
            processing: x.processing, 
            nodes: x.nodes.iter().map(|n| n.id()).collect_vec()
        }
    };
    let mut task_intervals: Vec<PropagatedTaskInterval> = 
        task_interval::find_task_intervals(graph[node.id()].machine_id().unwrap(), graph)
        .into_iter()
        .map(interval_to_ref)
        .collect_vec();
    
    task_maintenance(node, 100, &mut task_intervals, graph);
}

fn task_maintenance<T: Graph>(node: &impl NodeId, new_lower: u32, task_intervals: &mut Vec<PropagatedTaskInterval>, graph: &mut T) where T::Node: ConstrainedNode {
    let old_lower = graph[node.id()].est();
    
    debug_assert!(new_lower > old_lower);

    task_intervals.drain_filter(|ti| {
        if graph[ti.lower].est() < old_lower {
            return false;
        }
        if old_lower < graph[ti.upper].est() && graph[ti.upper].est() <= new_lower && ti.upper != ti.lower {
            return true;
        }

        unimplemented!("Reducing task intervals needs to be implemented");

    });
    for x in task_intervals.iter_mut() {
        x.nodes = vec!();
    }

    graph[node.id()].set_est(new_lower);

    // All taskintervals that start between old_lower and new_lower
    
}

