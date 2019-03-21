use disjunctgraph::{ Graph, GraphNode };

#[derive(Debug)]
pub enum ConstraintError {
    Infeasible
}

#[derive(Clone, Debug)]
pub struct ActivityConstraint {    
    pub left_bound: u32,
    pub right_bound: u32,
}

#[derive(Clone, Debug)]
pub struct ProblemConstraints {
    pub upper_bound: u32,
    pub constraints: Vec<ActivityConstraint>,
}

impl ProblemConstraints {
    pub fn new<I: Graph<J>, J: GraphNode>(graph: &I, upper_bound: u32) -> Result<Self, ConstraintError> {
        
        let topology = {
            let topology = graph.topology();
            let mut nodes = (0..graph.nodes().len()).collect::<Vec<_>>();
            nodes.sort_by_key(|x| std::cmp::Reverse(topology[*x]));
            nodes
        };        
        let nodes = graph.nodes();
        
        let mut constraints: Vec<ActivityConstraint> = vec!(ActivityConstraint { left_bound: 0, right_bound: 0 }; nodes.len());

        for node in topology.iter().map(|x| &nodes[*x]) {
            constraints[node.id()].left_bound = graph.predecessors(node).iter()
                    .map(|x| constraints[x.id()].left_bound + x.weight())
                    .max().unwrap_or(0);             
        }

        for node in topology.iter().rev().map(|x| &nodes[*x]) {
            let job_predecessor = graph.successors(node).iter()
                    .map(|x| constraints[x.id()].right_bound - x.weight())
                    .min().unwrap_or(upper_bound);
            
            if job_predecessor < constraints[node.id()].left_bound + node.weight() {
                return Err(ConstraintError::Infeasible);
            }
            constraints[node.id()].right_bound = job_predecessor;
        }

        Ok(ProblemConstraints { upper_bound, constraints })
    }

    
    pub fn check_precedence(&self, node_1: &impl GraphNode, node_2: &impl GraphNode) -> bool {
        // Check if arc node_1 -> node_2 is possible.
        // Meaning the earliest end date of node_1 has to be before the latest start date of node_2        
        let earliest_end = self.constraints[node_1.id()].left_bound + node_1.weight();
        let latest_end = self.constraints[node_2.id()].right_bound - node_2.weight();

        earliest_end <= latest_end
    }

    pub fn set_upper_bound(&mut self, upper_bound: u32) -> Result<(), ConstraintError> {
        unimplemented!()
    }
}