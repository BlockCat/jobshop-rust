use itertools::Itertools;
use disjunctgraph::{ ConstrainedNode, Graph, GraphNode, NodeId };

#[derive(Debug)]
pub struct TaskInterval<'a, T: Graph> where T::Node: ConstrainedNode + std::fmt::Debug {    
    pub upper: &'a T::Node,
    pub lower: &'a T::Node,
    pub processing: u32,
    pub nc_start: Vec<&'a T::Node>,
    pub nc_end: Vec<&'a T::Node>,
    pub nodes: Vec<&'a T::Node>,
}

impl<'a, T: Graph> std::fmt::Display for TaskInterval<'a, T> where T::Node: ConstrainedNode + std::fmt::Debug {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "(l: {}, u:{} -> start: {:?}, end: {:?}, nodes: {:?})", 
            self.lower(), 
            self.upper(), 
            self.nc_start.iter().map(|x| x.id()).collect_vec(),
            self.nc_end.iter().map(|x| x.id()).collect_vec(),
            self.nodes.iter().map(|x| x.id()).collect_vec())
    }
}

impl<'a, T: Graph> TaskInterval<'a, T> where T::Node: ConstrainedNode + std::fmt::Debug {
    
    pub fn slack(&self) -> u32 {
        self.upper() - self.lower() - self.processing
    }

    /// In this task interval, there should be enough space to execute all operations
    /// upper - lower >= processing
    fn feasible(&self) -> bool {
        self.upper() >= self.lower() + self.processing
    }

    pub fn upper(&self) -> u32 { self.upper.lct() }
    pub fn lower(&self) -> u32 { self.lower.est() }

    pub fn from_interval<'b>(graph: &T, resource: &[&'b T::Node], lower: &'b T::Node, upper: &'b T::Node) -> Option<TaskInterval<'b, T>> {

        // Task intervals should contain operations that have no disjunctions left        
        let nodes: Vec<&T::Node> = resource.iter()
            .filter(|node| node.est() >= lower.est() && node.lct() <= upper.lct())
            .cloned().collect();
        

        // TaskIntervals must have by definition 2 or more nodes.
        if nodes.len() < 2 {
            return None;
        }

        let processing = nodes.iter().map(|x| x.weight()).sum();

        
        // NC start is the set of jobs that can be executed before all jobs in S. This is the edge finding part in the paper:
        // If t was the first in the interval, is there enough space to execute the other nodes?
        // if it has any "node -> other" then node can't be first 
        let nc_start = nodes.iter()
            .filter(|node| upper.lct() >= node.est() + processing)
            .filter(|node| !nodes.iter().any(|other| graph.has_precedence(*other, **node)))
            .map(|x| *x)
            .collect::<Vec<_>>();
        
        // NC end is the set of jobs that can be executed after all jobs in S.        
        // If t was the last interval, is there enough space to execute the other nodes?
        // if it has any "other -> node" then node can't be last
        let nc_end = nodes.iter()
            .filter(|node| node.lct() >= lower.est() + processing)
            .filter(|node| !nodes.iter().any(|other| graph.has_precedence(**node, *other))) 
            .map(|x| *x)
            .collect::<Vec<_>>();

        let ti = TaskInterval {
            upper: upper, 
            lower: lower,
            processing, nodes,
            nc_start, nc_end
        };

        debug_assert!(ti.nc_start.len() >= 1, "An interval can always have a first node: {}\n {:?}", ti.nodes.len(), ti.nodes);
        debug_assert!(ti.nc_end.len() >= 1, "An interval can always have a last node");
        debug_assert!(ti.nc_start.iter().combinations(2).all(|x| graph.has_disjunction(*x[0], *x[1])), 
            "All nodes within nc_start should have disjunctions between each other, {}", ti.nc_start.len());
       
        debug_assert!(ti.nc_end.iter().combinations(2).all(|x| graph.has_disjunction(*x[0], *x[1])),
            "All nodes within nc_end should have disjunctions between each other, {}", ti.nc_start.len());
        debug_assert!(ti.feasible(), "Failed feasible");

        Some(ti)
    }
}

pub fn find_task_intervals<T: Graph>(resource: u32, graph: &T) -> Vec<TaskInterval<T>> where T::Node: ConstrainedNode + std::fmt::Debug {
    
    let operations = graph.nodes().iter().filter(|x| x.machine_id() == Some(resource)).collect_vec();
    let ests: Vec<&T::Node> = operations.iter().unique_by(|s| s.est()).sorted_by_key(|s| s.est()).cloned().collect_vec();
    let lcts: Vec<&T::Node> = operations.iter().unique_by(|s| s.lct()).sorted_by_key(|s| s.lct()).cloned().collect_vec();

    let mut j = 0;
    let mut task_intervals: Vec<TaskInterval<T>> = Vec::with_capacity(operations.len().pow(2));

    for i in 0..ests.len() {
        let lower = ests[i];
        while lcts[j].lct() < lower.est() {
            j += 1;
        }
        for j in j..lcts.len() {
            let upper = lcts[j];
            if lower.est() < upper.lct() {
                if let Some(ti) = TaskInterval::from_interval(graph, &operations, lower, upper) {
                    task_intervals.push(ti);
                }
            }
        }
    }
    
    task_intervals
}

/*
pub fn propagate_task_interval<T>(resources: &[usize], graph: &mut T) where T::Node: ConstrainedNode + std::fmt::Debug {
    
}*/