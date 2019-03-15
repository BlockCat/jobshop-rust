use disjunctgraph::{ Graph };

use crate::problem::{ ProblemSolver, Problem, ProblemNode, ProblemGraph };
use crate::schedule::Schedule;

// In the case of a search, it might be nice to only store partial orientations.
// As described in https://pure.tue.nl/ws/files/2119953/385216.pdf
struct LocalSearch {}

type LinkedGraph = disjunctgraph::LinkedGraph<ProblemNode>;

impl ProblemSolver for LocalSearch {
    type Solution = LinkedGraph;

    fn solve(problem: &Problem) -> Self::Solution {        
        let graph: ProblemGraph<LinkedGraph> = problem.into();
        let graph = graph.0;
        let (critical_length, critical_path) = graph.critical_path().unwrap();

        let mut temperature = 10;
        
        unimplemented!()
    }

}


