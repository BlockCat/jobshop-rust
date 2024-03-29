
use disjunctgraph::{ Graph, ConstrainedNode, GraphNode, NodeId };
use itertools::Itertools;
use std::collections::HashSet;

use crate::task_interval;


pub fn propagate_tail<I: Graph>(node: &impl NodeId, graph: &mut I, upper_bound: u32) -> Result<HashSet<usize>, String> where I::Node: ConstrainedNode {
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
                return Err(format!("Adjusting tail of {} to {} would lead to infeasability", id, min_tail));
            }
        }
    }

    Ok(changed)
}


pub fn propagate_head<I: Graph>(node: &impl NodeId, graph: &mut I, upper_bound: u32) -> Result<HashSet<usize>, String> where I::Node: ConstrainedNode {
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
                return Err(format!("Adjusting head of {} to {} would lead to infeasability", id, min_head));
            }
        }
    }

    Ok(changed)
}

pub fn edge_finding<I: Graph + std::fmt::Debug>(resource: u32, graph: &mut I, upper_bound: u32) -> Result<(), String> where I::Node: ConstrainedNode + std::fmt::Debug {

    let tis = task_interval::find_task_intervals(resource, graph, upper_bound)?;

    let starts = tis.iter()
        .filter(|ti| ti.nc_start.len() == 1)
        .flat_map(|ti| {
            let id = ti.nc_start[0].id();
            ti.nodes.iter()
                .filter(|n| graph.has_disjunction(&n.id(), &id))
                .map(|n| (id, n.id()))
                .collect_vec()
        });

    let ends = tis.iter()
        .filter(|ti| ti.nc_end.len() == 1)
        .flat_map(|ti| {
            let id = ti.nc_end[0].id();
            ti.nodes.iter()
                .filter(|n| graph.has_disjunction(&n.id(), &id))
                .map(|n| (n.id(), id))
                .collect_vec()
        });

    let fixations = starts.chain(ends).unique().collect_vec();

    for (other, end) in fixations {
        graph.fix_disjunction(&other, &end).or_else(|e| Err(format!("Could not fix disjunction {} -> {}, {:?}", other, end, e)))?;
        adjust_head_tail(graph, &other, &end, upper_bound)?;
    }

    Ok(())
}

fn adjust_head_tail<I: Graph>(graph: &mut I, node_1: &impl NodeId, node_2: &impl NodeId, upper_bound: u32) -> Result<(), String> where I::Node: ConstrainedNode {
    let node_1 = node_1.id();
    let node_2 = node_2.id();
    let old_tail = graph[node_1].tail();
    let old_head = graph[node_2].head();
    let new_tail = std::cmp::max(graph[node_2].tail() + graph[node_2].weight(), old_tail);
    let new_head = std::cmp::max(graph[node_1].head() + graph[node_1].weight(), old_head);

    if new_tail > old_tail {
        graph[node_1].set_tail(new_tail);
        propagate_tail(&node_1, graph, upper_bound)?;
        debug_assert!(graph.successors(&node_1).all(|o| o.tail() + o.weight() <= new_tail));
        debug_assert!(graph.predecessors(&node_1).all(|o| o.tail() + graph[node_1].weight() >= new_tail));
    }

    if new_head > old_head {
        graph[node_2].set_head(new_head);
        propagate_head(&node_2, graph, upper_bound)?;
        debug_assert!(graph.predecessors(&node_2).all(|o| o.head() + o.weight() <= new_head));
        debug_assert!(graph.successors(&node_2).all(|o| o.head() + graph[node_2].weight() >= new_head));
    }

    Ok(())
}
/// Propagate a fixation node_1 -> node_2
pub fn propagate_fixation<I: Graph + std::fmt::Debug>(graph: &mut I, node_1: &impl NodeId, node_2: &impl NodeId, upper_bound: u32) -> Result<(), String> where I::Node: ConstrainedNode + std::fmt::Debug {
    
    adjust_head_tail(graph, node_1, node_2, upper_bound)?;

    while search_orders(graph, upper_bound)? {
        for resource in 1..=10 {
            edge_finding(resource, graph, upper_bound)?;            
        }
    }

    debug_assert!(graph.nodes().iter().all(|node|{
            let current_head = node.head();
            let current_tail = node.tail();                
            let head = graph.predecessors(node).map(|o| o.head() + o.weight()).max().unwrap_or(0);
            let tail = graph.successors(node).map(|o| o.tail() + o.weight()).max().unwrap_or(0);

            assert!(node.head() + node.weight() + node.tail() <= upper_bound);
            assert!(current_head >= head, "wrong head propagation: {} >= {}", current_head, head);
            assert!(current_tail >= tail, "wrong tail propagation: {} >= {}", current_tail, tail);
            true
    }));

    Ok(())
}


pub fn search_orders<T: Graph + std::fmt::Debug> (graph: &mut T, upper_bound: u32) -> Result<bool, String> where T::Node: ConstrainedNode {
    
    let mut change_occured = false;
    for node in graph.nodes().iter().map(|n| n.id()).collect::<Vec<_>>() {
        for other in graph.disjunctions(&node).map(|n| n.id()).collect::<Vec<_>>() {

            let processing = graph[node].weight() + graph[other].weight();
            let node_other = graph[node].head() + processing + graph[other].tail();
            let other_node = graph[other].head() + processing + graph[node].tail();
            
            // If node -> other is bigger than the upper bound
            if node_other > upper_bound && other_node > upper_bound {                
                return Err(format!("Infeasible, nodes {} and {} can't be ordered {} > {} and {} > {}", node, other, node_other, upper_bound, other_node, upper_bound));
            }
            debug_assert!(node_other <= upper_bound || other_node <= upper_bound);
            
            let node = node.id();

            if node_other > upper_bound {
                
                debug_assert!(other_node <= upper_bound);                                               
                change_occured = true;                        
                graph.fix_disjunction(&other, &node).or(Err(format!("Could not fix disjunction {} -> {}", other.id(), node.id())))?;     
                adjust_head_tail(graph, &other, &node, upper_bound)?;
                                
            } else if other_node > upper_bound {                
                debug_assert!(node_other <= upper_bound);
                change_occured = true;
                graph.fix_disjunction(&node, &other).or(Err(format!("Could not fix disjunction {} -> {}", node.id(), other.id())))?;
                adjust_head_tail(graph, &node, &other, upper_bound)?;
            }
        }
    }

    Ok(change_occured)
}


// What do we need to do: 
// The paper tells us first about propagation,
// then interval maintenance
// these two are interleaved, maintenance triggers propagation
// and propagation triggers maintenance
