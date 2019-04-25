use disjunctgraph::{ Graph, GraphNode, NodeId };
use hashbrown::HashSet;
use std::collections::VecDeque;

use crate::problem::{ ProblemSolver, Problem, ProblemNode };

// In the case of a search, it might be nice to only store partial orientations.
// As described in https://pure.tue.nl/ws/files/2119953/385216.pdf
pub struct BranchAndBound;

type LinkedGraph = disjunctgraph::LinkedGraph<ProblemNode>;

impl BranchAndBound {
    pub fn new() -> Self {
        BranchAndBound {}
    }
}
impl ProblemSolver for BranchAndBound {
    type Solution = LinkedGraph;

    fn solve(&self, problem: &Problem) -> Self::Solution {
        // Calculate a solution using heuristics
        let mut upper_graph: LinkedGraph = crate::local_search::LocalSearch::new(2000).solve(problem);
        let mut upper_bound  = upper_graph.critical_length().unwrap();

        let mut stack = VecDeque::with_capacity(problem.activities.len() * 10);
        
        stack.push_back(problem.into_graph());

        while !stack.is_empty() {
            let current_node: LinkedGraph = stack.pop_back().unwrap();
            let new_upper_bound = calculate_upperbound(&current_node);
            
            if new_upper_bound.1 < upper_bound {
                upper_graph = new_upper_bound.0;
                upper_bound = new_upper_bound.1;
            }

            let (_, crit_path) = current_node.critical_path().unwrap();

            let blocks = calculate_blocks(crit_path);
            let before_sets = calculate_pre_set(&current_node, &blocks).into_iter().map(|x| (true, x));
            let after_sets = calculate_pre_set(&current_node, &blocks).into_iter().map(|x| (false, x));

            // Handle Before sets then after sets
            for (before, set) in before_sets.chain(after_sets) {
                for operation in set {
                    // Create a successor node
                    let successor = current_node.clone();
                    // Fix the disjunctions, let operation be in front of all its disjunctions
                    let successor = current_node
                        .disjunctions(operation)
                        .into_iter()
                        .fold(successor, |acc, disjunction| 
                            match before {
                                true => acc.fix_disjunction(operation, disjunction).unwrap(),
                                false => acc.fix_disjunction(disjunction, operation).unwrap(),
                            }                            
                        );

                    // Calculate lower bound
                    let lowerbound: u32 = calculate_lowerbound(&successor);
                    // Lowerbound is under UB, add to stack
                    if lowerbound < upper_bound {
                        stack.push_back(successor);
                    }
                }
            }
        }

        upper_graph
    }
}

fn calculate_blocks(crit_path: Vec<&ProblemNode>) -> Vec<Vec<&ProblemNode>> {
    use itertools::Itertools;

    crit_path.into_iter()
        .group_by(|x| x.machine_id())
        .into_iter()
        .map(|group| group.1.collect::<Vec<_>>  ())
        .filter(|group| group.len() > 1)
        .collect()
}

// Calculate E_j^B for j \in {0, ..., k}
fn calculate_pre_set<'a>(graph: &LinkedGraph, blocks: &Vec<Vec<&'a ProblemNode>>) -> Vec<HashSet<&'a ProblemNode>> {
    blocks.iter()
        .map(|group| 
            group.into_iter()
                .filter(|node| graph.disjunctions(**node).len() > 0)
                .cloned()
                .collect::<HashSet<_>>()
        ).collect()
}

fn calculate_lowerbound(graph: &LinkedGraph) -> u32 {
    unimplemented!()
}

fn calculate_upperbound(graph: &LinkedGraph) -> (LinkedGraph, u32) {
    unimplemented!()
}
