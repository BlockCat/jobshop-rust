use disjunctgraph::{ Graph, GraphNode, NodeId };

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
    pub fn new<I: Graph>(graph: &I, upper_bound: u32) -> Result<Self, ConstraintError> {
        
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

        let mut problem = ProblemConstraints { upper_bound, constraints };
        problem.fix_2b_consistency(graph);
        problem.fix_3b_consistency(graph);
        problem.fix_2b_consistency(graph);

        // For fun:
        for node in graph.nodes() {
            for disjunction in graph.disjunctions(node) {
                if !problem.check_precedence(node, disjunction) {
                    println!("Precedence found: {} -> {}", disjunction.id(), node.id());
                }
            }
        }
        Ok(problem)
    }

    fn fix_2b_consistency<I: Graph>(&mut self, graph: &I) {
        let nodes = graph.nodes();

        
        let mut should_continue = true;
        while should_continue {
            dbg!("This is inefficient === 2");
            for node in nodes {
                should_continue = self.fix_2b_consistency_node(graph, node);
            }
        }
    }

    fn fix_2b_consistency_node<I: Graph>(&mut self, graph: &I, node: &I::Node) -> bool {
        let mut changed = false;
        let p_i = node.weight();
        let est_i = self.constraints[node.id()].left_bound;
        let lst_i = self.constraints[node.id()].right_bound - p_i;        

        for disjunction in graph.disjunctions(node) {
            let p_j = disjunction.weight();
            let lst_j = self.constraints[disjunction.id()].right_bound - p_j;
            let est_j = self.constraints[disjunction.id()].left_bound;
            if (est_i + p_i > lst_j) & (est_j + p_j > est_i) { // Executes 2-b-consistency rule
                self.constraints[node.id()].left_bound = est_j + p_j;
                changed = true;
            }

            if (est_j + p_j > lst_i) & (lst_j < lst_i + p_i) {
                self.constraints[node.id()].right_bound = lst_j;
                changed = true;
            }
        }

        changed
    }

    fn fix_3b_consistency<I: Graph>(&mut self, graph: &I) {
        let nodes = graph.nodes();
        
        let mut should_continue = true;
        while should_continue {
            dbg!("This is inefficient === 3");
            for node in nodes {
                should_continue = self.fix_3b_consistency_node(graph, node);
            }
        }
    }

    // Assuming that it is 2b-consistent
    fn fix_3b_consistency_node<I: Graph>(&mut self, graph: &I, node: &I::Node) -> bool {
        let mut changed = false;  
        for j in graph.disjunctions(node) {
            for k in graph.disjunctions(node) {
                if j.id() > k.id() {
                    let (p_i, p_j, p_k) = (node.weight(), j.weight(), k.weight());
                    let est_i = self.constraints[node.id()].left_bound;
                    let est_j = self.constraints[j.id()].left_bound;
                    let est_k = self.constraints[k.id()].left_bound;

                    
                    let lct_j = self.constraints[j.id()].right_bound;
                    let lct_k = self.constraints[k.id()].right_bound;

                    let m = *([lct_j.saturating_sub(est_i), lct_j.saturating_sub(est_k), lct_k.saturating_sub(est_i), lct_k.saturating_sub(est_j)].into_iter().max().unwrap());
                    let next_value = std::cmp::min(est_j, est_k) + p_j + p_k;
                    if (m < p_i + p_j + p_k) & (next_value > est_i) {
                        self.constraints[node.id()].left_bound = next_value;
                        changed = true;
                    } else if ((est_i + p_i + p_j > lct_j) | (est_i + p_i + p_k > lct_k)) & (next_value > est_i) {
                        self.constraints[node.id()].left_bound = next_value;
                        changed = true;
                    }
                }
            }
        }

        changed
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