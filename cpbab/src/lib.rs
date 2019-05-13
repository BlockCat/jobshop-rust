extern crate disjunctgraph;

mod node;

use disjunctgraph::{ GraphNode, NodeId, Graph };

// Constrained graph ;)
type CGraph = disjunctgraph::LinkedGraph<node::Node>;

const PAR: u32 = 3;


pub fn branch_and_bound() {
    loop {

    }
}

struct TaskInterval<'a> {
    resource: usize,
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

    fn from_interval(resource: usize, lower: u32, upper: u32, graph: &CGraph) -> TaskInterval {

        let nodes: Vec<&node::Node> = graph.nodes().iter().filter(|node| {
            node.machine_id() == Some(resource as u32) &&
            node.est() >= lower && node.lct() <= upper
        }).collect();

        let processing = nodes.iter().map(|x| x.weight()).sum();

        TaskInterval {
            resource, upper, lower, processing, nodes,
            nc_start: unimplemented!(),
            nc_end: unimplemented!(),
        }
    }
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

/// Can order nodes t1 and t2: t1 -> t2 (t1 before t2)
fn check_precedence(t1: &node::Node, t2: &node::Node) -> bool {
    t1.est() + t1.weight() + t2.weight() <= t2.lct()
}

fn next_pair<'a>(resources: &[usize], graph: &'a CGraph, max_makespan: u32) -> (&'a node::Node, &'a node::Node) {

    // Calculate the critical task interval for each resource/machine
    let criticals: Vec<TaskInterval> = resources.iter().map(|id| crit(*id, graph)).collect();
    // Calculate the slack for each resource
    let resource_slacks: Vec<u32> = resources.iter().map(|id| resource_slack(*id as u32, graph)).collect(); 
    
    // Find the resource with the most constrained task interval
    let (crit, _) = criticals.iter().zip(resource_slacks).min_by_key(|(cr, rr)| {
        cr.slack() * rr * std::cmp::min(PAR, NC(cr) as u32)
    }).unwrap();

    
    let t1 = crit.nc_start.iter().min_by_key(|x| x.est()).expect("Could not extract left bound node"); // Get left bounded node on the task interval
    let t2 = crit.nc_end.iter().max_by_key(|x| x.lct()).expect("Could not extract right bound node"); // Get right bounded node on the task interval
    
    let crit_slack = crit.slack();

    let S1 = crit.nc_start.iter()
        .filter(|t| t.id() != t1.id())
        .filter(|t| t.est() <= t1.est() + crit_slack)
        .filter(|t| !check_precedence(t1, t))
    .collect::<Vec<_>>();
    
    let S2 = crit.nc_end.iter()
        .filter(|t| t.id() != t2.id())
        .filter(|t| t.lct() >= t2.lct() - crit_slack)
        .filter(|t| !check_precedence(t, t2))
    .collect::<Vec<_>>();

    if S1.len() <= S2.len() {
        //let dS = crit.nodes.iter().filter(|x| x.id() != t1.id()).map(|x| x.est()).min().unwrap().unwrap() - t1.est().unwrap();
        let t: &node::Node = S1.iter().min_by_key(|t| h(t1, t, max_makespan)).unwrap();

        if g(t1, t, max_makespan) <= g(t, t1, max_makespan) {
            (t1, t)
        } else {
            (t, t1)
        }
    } else {
        let t: &node::Node = S2.iter().min_by_key(|t| h(t, t2, max_makespan)).unwrap();

        if g(t, t2, max_makespan) <= g(t2, t, max_makespan) {
            (t, t2)
        } else {
            (t2, t)
        }
    }
}

fn h(t1: &node::Node, t2: &node::Node, max_makespan: u32) -> u32 {
    let t1b = g(t1, t2, max_makespan);
    let tb1 = g(t2, t1, max_makespan);

    let fff = unimplemented!("Take the change to S into account or something");

    if t1b > tb1 || t1b > fff { // Is it bigger than one of them
        t1b
    } else if tb1 > fff { // Smaller than both of them
        fff
    } else {
        tb1
    }    
}

fn g(t1: &node::Node, t2: &node::Node, allowed_makespan: u32) -> u32 {
    let da = std::cmp::max(0, t1.lct() - t2.lct() + t2.weight());
    let db = std::cmp::max(0, t1.est() + t1.weight() - t2.est());

    let a = evaluation(t1.lct() - t1.est() - t1.weight(), da, allowed_makespan);
    let b = evaluation(t2.lct() - t2.est() - t2.weight(), db, allowed_makespan);

    if a < b {
        a
    } else {
        b
    }
}

fn evaluation(Delta: u32, delta: u32, M: u32) -> u32 {
    if delta == 0 {
        M
    } else if Delta < delta {
        0
    } else {
        (Delta - delta).pow(2) / Delta
    }
}

fn crit<'a>(resource_id: usize, graph: &'a CGraph) -> TaskInterval<'a> {
    let task_intervals: Vec<TaskInterval<'a>> = unimplemented!();

    task_intervals.into_iter()        
        .min_by_key(|x| x.slack() * NC(x) as u32)
        .unwrap()
}

fn NC(task_interval: &TaskInterval) -> usize {
    std::cmp::min(task_interval.nc_start.len(),  task_interval.nc_end.len())
}

// According to: Adjustment of heads and tails for the job-shop problem (J. Carlier and E. Pinson)
// Chapter 4.4: Lower bound
fn lower_bound() -> u32 {
    unimplemented!()
}