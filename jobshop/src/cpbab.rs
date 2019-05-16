use disjunctgraph::{ Graph, GraphNode, NodeId };
use hashbrown::HashSet;
use std::collections::VecDeque;

use crate::problem::{ ProblemSolver, Problem, ProblemNode };

// In the case of a search, it might be nice to only store partial orientations.
// As described in https://pure.tue.nl/ws/files/2119953/385216.pdf
pub struct CPBAB;

type LinkedGraph = disjunctgraph::LinkedGraph<ProblemNode>;

impl CPBAB {
    pub fn new() -> Self {
        CPBAB {}
    }
}
impl ProblemSolver for CPBAB {
    type Solution = cpbab::CGraph;

    fn solve(&self, problem: &Problem) -> Self::Solution {
        /*let mm = crate::local_search::LocalSearch::new(100)
            .solve(problem)
            .critical_length().unwrap();*/
        let graph = problem.into_graph();
        cpbab::branch_and_bound(graph, problem.machines as usize, 13)
    }
}