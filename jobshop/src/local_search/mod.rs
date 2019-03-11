use disjunctgraph::{ Graph };

use crate::problem::{ ProblemSolver, Problem, ProblemNode };
use crate::schedule::Schedule;

// In the case of a search, it might be nice to only store partial orientations.
// As described in https://pure.tue.nl/ws/files/2119953/385216.pdf
struct LocalSearch {}

type MatrixGraph = disjunctgraph::MatrixGraph<ProblemNode>;

impl ProblemSolver for LocalSearch {

    fn solve(problem: &Problem) -> Schedule {        
        let graph: MatrixGraph = problem.into();
        let critical_path = graph.critical_path();
        
        unimplemented!()
    }

}


