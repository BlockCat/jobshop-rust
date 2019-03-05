use std::rc::Rc;

use crate::problem::Problem;

pub struct DisjunctiveGraph {
    nodes: Vec<Rc<Node>>,
    edges: Vec<Rc<Edge>>,
}

pub struct Node {
    machine_id: u32,
    processing_time: u32,
    edges: Vec<Edge>
}

pub enum Edge {
    Disjunction(DisjunctiveEdge),
    Directed(DirectedEdge)
}

pub struct DisjunctiveEdge {
    source: Rc<Node>,
    target: Rc<Node>
}

pub struct DirectedEdge {
    source: Rc<Node>,
    target: Rc<Node>
}

impl Into<DisjunctiveGraph> for &Problem {
    fn into(self) -> DisjunctiveGraph {
        // For every job:
        let nodes = self.jobs.iter()
            .flat_map(|activities| {
                // For every activity create a node
                activities.0.iter()                    
                    .map(|activity| {
                        Node {
                            machine_id: activity.machine_id,
                            processing_time: activity.process_time,
                            edges: Vec::new()
                        }
                    })
            });

        
        panic!()
    }
}