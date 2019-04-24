use disjunctgraph::{ Graph, GraphNode, NodeId };

use crate::problem::{ ProblemSolver, Problem, ProblemNode };

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
        use rand::seq::SliceRandom;

        let graph = problem.into_graph::<LinkedGraph>();
        
        let mut graph = graph.into_directed().expect("Graph was directed, something went wrong check code.");
        let mut no_improvement_cycles = 0;
        let mut counter = 0;
        let mut rand = rand::thread_rng();

        while no_improvement_cycles <= 3 {

            counter += 1;

            let (critical_length, critical_path) = graph.critical_path().expect("Cyclic graph");
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
                let candidate_length = candidate_graph.critical_length().unwrap();

                if candidate_length < critical_length + temperature {
                    graph = candidate_graph;
                    improvement_found = true;
                    break;
                } else {
                    // Turn the candidate back around
                    graph = candidate_graph.flip_edge(&b, &a).expect("Could not return the flipped edge back")
                }
            }

            if improvement_found {
                no_improvement_cycles = 0;
            } else {
                no_improvement_cycles += 1;
            }
        }

        graph
    }

}

