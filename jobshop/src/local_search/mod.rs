use disjunctgraph::{ Graph, GraphNode, NodeId };

use crate::problem::{ ProblemSolver, Problem, ProblemNode, ProblemGraph };
use crate::schedule::Schedule;

// In the case of a search, it might be nice to only store partial orientations.
// As described in https://pure.tue.nl/ws/files/2119953/385216.pdf
pub struct LocalSearch {    
    temperature: u32,
}

type LinkedGraph = disjunctgraph::LinkedGraph<ProblemNode>;

impl LocalSearch {
    pub fn new(temperature: u32) -> Self {
        LocalSearch {
            temperature
        }
    }
}
impl ProblemSolver for LocalSearch {
    type Solution = LinkedGraph;

    fn solve(&self, problem: &Problem) -> Self::Solution {        
        use rand::Rng;
        use rand::seq::SliceRandom;

        let graph = ProblemGraph::<LinkedGraph, ProblemNode>::from(problem).0;
        
        let mut graph = graph.into_directed().expect("Graph was directed, something wentwrong check code.");
        let mut waiter = 0;
        let mut counter = 0;
        let mut rand = rand::thread_rng();

        while waiter <= 10 {

            counter += 1;

            let (critical_length, critical_path) = graph.force_critical_path();            
            let temperature = self.temperature / counter; // -T ln u (u \in Normal(1,0), T decreases)
            
            // Find multiple candidate switches
            let mut candidates = critical_path.windows(2).filter_map(|x| {
                let a = x[0];
                let b = x[1];

                if a.job_id() != b.job_id() && a.machine_id() == b.machine_id() {
                    Some((a.id(), b.id()))
                } else {
                    None
                }
            }).collect::<Vec<_>>();

            candidates.shuffle(&mut rand);            

            let mut improvement_found = false;

            for (a, b) in candidates {
                let candidate_graph = graph.flip_edge(&a, &b).expect("Could not flip edge.");
                let (candidate_length, _) = candidate_graph.force_critical_path();

                if candidate_length < critical_length + temperature {
                    graph = candidate_graph;
                    improvement_found = true;
                    break;
                }
            }

            if improvement_found {
                waiter = 0;
            } else {
                waiter += 1;
            }
        }

        graph
    }

}


