use crate::problem::{ ProblemSolver, Problem };
use disjunctgraph::Graph;

// In the case of a search, it might be nice to only store partial orientations.
// As described in https://pure.tue.nl/ws/files/2119953/385216.pdf
pub struct CPBAB;

impl CPBAB {
    pub fn new() -> Self {
        CPBAB {}
    }
}
impl ProblemSolver for CPBAB {
    type Solution = cpbab::CGraph;

    fn solve(&self, problem: &Problem) -> Self::Solution {
        let mm = crate::local_search::LocalSearch::new(5000)
            .solve(problem)
            .critical_length().unwrap();
        let graph = problem.into_graph();

        println!("Found local search: {}", mm);
        
        let solution = cpbab::branch_and_bound(graph, problem.machines as usize, 578);//found 579;

        println!("dbg: {:?}", solution);
        
        solution

    }
}