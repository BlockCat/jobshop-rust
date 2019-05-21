extern crate disjunctgraph;

mod node;

use std::collections::VecDeque;

use disjunctgraph::{ GraphNode, ConstrainedNode, NodeId, Graph };
use itertools::Itertools;

// Constrained graph ;
pub type CGraph = disjunctgraph::LinkedGraph<node::Node>;

const PAR: u32 = 3;

// What is still needed?
// There is no propagation of constraints,
// Operations that have no disjunctions left are looked at too.
// I believe cycles can occur? ()
pub fn branch_and_bound(mut root: CGraph, resources: usize, max_makespan: u32) -> CGraph {
    root.init_weights(max_makespan).expect("Problem with makespan is not feasible");    
    root.search_orders();
    
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
            let length = node.critical_length().expect("Could not calculate critical length");
            if length <= upper_bound {
                upper_bound = length;
                current_best = node;
            }
        } else {
            for (t1, t2) in next_pair(&resources, &node, upper_bound) {
                let mut graph = node.clone().fix_disjunction(t1, t2).expect("Could not fix disjunction");
                if graph.propagate(t1, t2).is_ok() {                    
                    if dbg!(lower_bound(&graph, max_makespan, &resources)) <= upper_bound {
                        stack.push_front(graph);
                    }
                } else {
                }
            }
        }
    }

    current_best
}

#[derive(Debug)]
struct TaskInterval<'a> {    
    upper: u32,
    lower: u32,
    processing: u32,
    nc_start: Vec<&'a node::Node>,
    nc_end: Vec<&'a node::Node>,
    nodes: Vec<&'a node::Node>,
}

impl<'a> TaskInterval<'a> {
    
    fn slack(&self) -> u32 {
        self.upper - self.lower - self.processing
    }

    /// In this task interval, there should be enough space to execute all operations
    /// upper - lower >= processing
    fn feasible(&self) -> bool {
        self.upper >= self.lower + self.processing
    }

    fn from_interval<'b>(graph: &CGraph, resource: &[&'b node::Node], lower: &node::Node, upper: &node::Node) -> Option<TaskInterval<'b>> {

        // Task intervals should contain operations that have no disjunctions left        
        let nodes: Vec<&node::Node> = resource.iter()
            .filter(|node| node.est() >= lower.est() && node.lct() <= upper.lct())
            .cloned().collect();
        

        // TaskIntervals must have by definition 2 or more nodes.
        if nodes.len() < 2 {
            return None;
        }

        // If precedence: {lower -> n | n \in nodes} for all
        // If precedence: {n -> upper | n \in nodes} for all
        // if these two checks are true then there is nothing really to check.        
        if nodes.iter().all(|n| graph.has_precedence(lower, *n) && graph.has_precedence(*n, upper)) 
        {
            return None;
        }        

        let processing = nodes.iter().map(|x| x.weight()).sum();

        // Calculate nc_start: 
        // NC start is the set of jobs that can be executed before all jobs in S. This is the edge finding part in the paper:
        // say we have a job t, if up(S) - low(t) - p(S) >= 0 => t can be first
        // up(S) - low(t) - p(S) >= 0 => up(S) >= low(t) + p(S)
        // If t was the first in the interval, is there enough space to execute the other nodes?

        let nc_start = nodes.iter()
            .filter(|node| upper.lct() >= node.est() + processing);
            

        // Keep nodes that have no predecessor on the same resource
        // if it has any "other -> node" then it's a nope
        let nc_start = nc_start
            .filter(|node| !nodes.iter()
                .filter(|o| node.id() != o.id())
                .any(|other| graph.has_precedence(*other, **node)))
            .map(|x| *x)
            .collect::<Vec<_>>();


        // All nodes within nc_start should be in disjunction to each other.
        // If a pair of nodes (a, b) was not in disjunction with each other, 
        // then (a -> b) and b could not be first, or (b -> a) and a could not be first.
        // Likewise for nc_end.
        
        
        // Calculate nc_end
        // NC end is the set of jobs that can be executed after all jobs in S.
        // we have job t: up(t) - p(S) >= low(S) => t can be last
        // up(t) - p(S) >= low(S) ========> up(t) >= low(S) + p(S)
        // If t was the last interval, is there enough space to execute the other nodes?        
        let nc_end = nodes.iter()
            .filter(|node| node.lct() >= lower.est() + processing);
            

        // Keep nodes that have no successor on the same resource 
        // if it has any "node -> other" then it's a nope
        let nc_end = nc_end
            .filter(|node| !nodes.iter().any(|other| graph.has_precedence(**node, *other)))
            .map(|x| *x)
            .collect::<Vec<_>>();

        let ti = TaskInterval {
            upper: upper.lct(), 
            lower: lower.est(),
            processing, nodes,
            nc_start, nc_end
        };

        debug_assert!(ti.nc_start.len() >= 1, "An interval can always have a first node: {}\n {:?}", ti.nodes.len(), ti.nodes);
        debug_assert!(ti.nc_end.len() >= 1, "An interval can always have a last node");
        debug_assert!(ti.nc_start.iter().combinations(2).all(|x| graph.has_disjunction(*x[0], *x[1])), 
            "All nodes within nc_start should have disjunctions between each other, \nnodes: {:?}\n\n graph: {:?}", ti.nc_start, graph, );
       
        debug_assert!(ti.nc_end.iter().combinations(2).all(|x| graph.has_disjunction(*x[0], *x[1])),
            "All nodes within nc_end should have disjunctions between each other, {}", ti.nc_start.len());
        debug_assert!(ti.feasible(), "Failed feasible: {:?}", ti);

        Some(ti)
    }
}

fn next_pair<'a>(resources: &[usize], graph: &'a CGraph, max_makespan: u32) -> Vec<(&'a node::Node, &'a node::Node)> {

    // Calculate the critical task interval for each resource/machine
    // Returns true if machine still has operations that need to be ordered
    let resource_filter = |id: &&usize| -> bool {
        graph.nodes().iter()
            .filter(|x| x.machine_id() == Some(**id as u32))
            .any(|x| graph.node_has_disjunction(x))
    };
    let criticals: Vec<Option<TaskInterval>> = resources.iter()
        .filter(resource_filter)
        .map(|id| crit(*id, graph))
        .collect();
    

    // Calculate the slack for each resource
    let resource_slacks: Vec<u32> = resources.iter().map(|id| resource_slack(*id as u32, graph)).collect(); 
    
    // Find the resource with the most constrained task interval
    let (crit, _) = criticals.iter().zip(resource_slacks)        
        .filter_map(|(cr, rr)| {
            if let Some(cr) = cr {
                Some((cr, cr.slack() as u32 * rr * std::cmp::min(PAR, num_choices(cr) as u32)))
            } else {
                None
            }
        })
        //.inspect(|p| println!("cr: {:?}", p.1))
        //.filter(|(_, rr)| *rr >= 0)
        .min_by_key(|(_, rr)| *rr)
        .expect("Can't find critical");

    debug_assert!(!crit.nodes.iter().all(|x| !graph.node_has_disjunction(*x)));

    let t1 = crit.nc_start.iter().min_by_key(|x| x.est()).expect("Could not extract left bound node"); // Get left bounded node on the task interval
    let t2 = crit.nc_end.iter().max_by_key(|x| x.lct()).expect("Could not extract right bound node"); // Get right bounded node on the task interval
    
    
    let s1 = crit.nc_start.iter().filter(|x| x.id() != t1.id()).collect_vec();
    let s2 = crit.nc_end.iter().filter(|x| x.id() != t2.id()).collect_vec();

    println!("{} <= {}", s1.len(), s2.len());
    // 0 <= 0, this means there

    debug_assert!(s1.len() > 0 || s2.len() > 0);

    if s1.len() <= s2.len() && s1.len() > 0 {
        let delta = crit.nodes.iter()
            .filter(|x| x.id() != t1.id())
            .map(|x| x.est()).min().expect("No min S1 found") - t1.est();

        let t: &node::Node = s1.iter()
            .min_by_key(|t| left_bounded_entropy(t1, t, max_makespan, crit, delta))
            .expect("Could not minimize h1");

        if g(t1, t, max_makespan) <= g(t, t1, max_makespan) {
            vec!((t1, t), (t, t1))
        } else {
            vec!((t, t1), (t1, t))
        }
    } else {
        let delta = t2.lct() - crit.nodes.iter()
            .filter(|x| x.id() != t2.id())
            .map(|x| x.lct()).max().expect("No max S2 found");
        let t: &node::Node = s2.iter()
            .min_by_key(|t| right_bounded_entropy(t, t2, max_makespan, crit, delta))
            .expect("Could not minimize h2");

        if g(t, t2, max_makespan) <= g(t2, t, max_makespan) {
            vec!((t, t2), (t2, t))
        } else {
            vec!((t2, t), (t, t2))
        }
    }
}


/// Find the critical on a resource, if there is no found then there inconsistency
/// It can happen that a resource is already completely scheduled.
fn crit<'a>(resource_id: usize, graph: &'a CGraph) -> Option<TaskInterval<'a>> {
    
    // Get the nodes on the resources    
    let resource = graph.nodes().iter()
        .filter(|n| n.machine_id() == Some(resource_id as u32))        
        .collect_vec();

    

    // TaskIntervals can have a node in front or after all the nodes in the taskinterval, but these scheduled nodes should
    // not be part of the definition of the interval
    let ests = resource.iter().sorted_by_key(|n| n.est()).collect::<Vec<_>>();
    let lcts = resource.iter().sorted_by_key(|n| n.lct()).collect::<Vec<_>>();
    
    let mut j = 0;

    let mut task_intervals: Vec<TaskInterval<'a>> = Vec::with_capacity(resource.len().pow(2));

    for i in 0..ests.len() {
        let lower = ests[i];
        while lcts[j].lct() < lower.est() {
            j += 1;
        }
        for j in j..lcts.len() {
            let upper = lcts[j];
            if lower.est() < upper.lct() {
                if let Some(ti) = TaskInterval::from_interval(graph, &resource, lower, upper) {
                    task_intervals.push(ti);
                }
            }
        }
    }

    // No interesting task intervals found on this resource
    if task_intervals.len() == 0 {
        return None;
    }

    // debug_assert!(task_intervals.len() >= 1, "Not enough tasks found: {}\necsts: {:?}\n lcts: {:?}", 
    //     task_intervals.len(),
    //     ests,
    //     lcts
    // );

    // Only resources are considered that have more than 1 node anyway.
    task_intervals.into_iter()
        .filter(|x| x.nodes.len() > 1)
        //.inspect(|x| println!("\n\n{} - task intervals: {:?}", resource_id, x))
        .min_by_key(|x| x.slack() as u32 * num_choices(x) as u32)
}

/// Calculate the slack for all operations on a resource
fn resource_slack(resource: u32, graph: &CGraph) -> u32 {
    let (min, max, p) = graph.nodes().iter()
        .filter(|x| x.machine_id() == Some(resource as u32))
        .fold((std::u32::MAX, 0, 0), |(min, max, p), x| {
            let min = std::cmp::min(min, x.est());
            let max = std::cmp::max(max, x.lct());
            let p = p + x.weight();
            (min, max, p)
        });
    max - min - p
}


/// When fixing t1 -> t2, this function returns the expected reduction in entropy on the domain
fn left_bounded_entropy(t1: &node::Node, t2: &node::Node, max_makespan: u32, task_interval: &TaskInterval, delta: u32) -> u32 {
    let t1_tb = g(t1, t2, max_makespan);
    let tb_t1 = g(t2, t1, max_makespan);

    // If this assert fails then that means that t2 cannot be placed to the left
    debug_assert!(task_interval.upper >= t2.est() + task_interval.processing);

    let new_slack = task_interval.upper - t2.est() - task_interval.processing;
    let fff = evaluation(new_slack, delta, max_makespan);

    std::cmp::max(t1_tb, std::cmp::min(tb_t1, fff))
}

fn right_bounded_entropy(t1: &node::Node, t2: &node::Node, max_makespan: u32, task_interval: &TaskInterval, delta: u32) -> u32 {
    let t1_tb = g(t1, t2, max_makespan);
    let tb_t1 = g(t2, t1, max_makespan);

    // If this assert fails then that means that t2 cannot be placed
    debug_assert!(t2.lct() >= task_interval.lower + task_interval.processing);

    let new_slack = t2.lct() - task_interval.lower - task_interval.processing;
    let fff = evaluation(new_slack, delta, max_makespan);

    std::cmp::max(t1_tb, std::cmp::min(tb_t1, fff))
}

fn g(t1: &node::Node, t2: &node::Node, allowed_makespan: u32) -> u32 {
    let da = (t1.lct() + t2.weight()).saturating_sub(t2.lct());
    let db = (t1.est() + t1.weight()).saturating_sub(t2.est());
    

    let a = evaluation(t1.lct() - t1.est() - t1.weight(), da, allowed_makespan);
    let b = evaluation(t2.lct() - t2.est() - t2.weight(), db, allowed_makespan);

    if a < b {
        a
    } else {
        b
    }
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
fn lower_bound(graph: &CGraph, max_makespan: u32, resources: &[usize]) -> u32 {    
    let resources = resources.iter()
        .map(|resource| graph.nodes().iter().filter(move |n| n.machine_id() == Some(*resource as u32))) // Returns an I_k on machine M_k
        .collect::<Vec<_>>();
        
    let d1 = resources.into_iter()
        .flat_map(|resource| resource.combinations(2)) // Take all combinations of 2
        .map(|comb| {
            let n1 = &comb[0];
            let n2 = &comb[1];

            let a = n1.est() + n1.weight() + n2.weight() + (max_makespan - n2.lct());
            let b = n2.est() + n2.weight() + n1.weight() + (max_makespan - n1.lct());
            std::cmp::max(a, b)
        })        
        .filter(|d| d <= &max_makespan)
        .max().unwrap_or(max_makespan + 1);
    d1 - 1
}
