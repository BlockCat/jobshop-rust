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
        
        let topology = graph.topology1().collect::<Vec<&I::Node>>();
        let nodes = graph.nodes();
        
        let mut constraints: Vec<ActivityConstraint> = vec!(ActivityConstraint { left_bound: 0, right_bound: 0 }; nodes.len());

        for node in topology.iter() {
            constraints[node.id()].left_bound = graph.predecessors(*node).iter()
                    .map(|x| constraints[x.id()].left_bound + x.weight())
                    .max().unwrap_or(0);             
        }

        for node in topology.iter().rev() {
            let job_predecessor = graph.successors(*node).iter()
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
        println!("Is it temp 3b consistent: {}", problem.check_3b_precedence(graph));
        problem.fix_2b_consistency(graph);

        // For fun:graph.fix_disjunction(disjunction, node).expect("Could not fix");

        nodes.into_iter()
            .map(|x| graph.disjunctions(x).into_iter().map(move |y| (x, y)))
            .flatten()
            .filter(|&(x, y)| !problem.check_precedence(x, y))            
            .map(|(x, y)| (y.id(), x.id()))
            .for_each(|(x, y)| println!("Found dependency: {} -> {}", x, y));

        Ok(problem)
    }

    fn fix_2b_consistency<I: Graph>(&mut self, graph: &I) {
        let nodes = graph.nodes();
        
        dbg!("This is inefficient, fix_2b_consistency");
        while nodes.iter().map(|node| self.fix_2b_consistency_node(graph, node)).any(|x| x) {
            // Do nothing,
        }
        
    }

    fn fix_2b_consistency_node<I: Graph>(&mut self, graph: &I, node: &I::Node) -> bool {
        let mut changed = false;

        for disjunction in graph.disjunctions(node) {
            let p_j = disjunction.weight();
            let lst_j = self.constraints[disjunction.id()].right_bound - p_j;
            let est_j = self.constraints[disjunction.id()].left_bound;

            // Node -> disjunction not possible.
            // Has to be disjunction -> node
            if !self.check_precedence(node, disjunction) {
                self.constraints[node.id()].left_bound = est_j + p_j;
                changed = true;
            }

            // Disjunction -> node not possible,
            // has to be node -> disjunction
            if !self.check_precedence(disjunction, node) {
                self.constraints[node.id()].right_bound = lst_j;
                changed = true;
            }
        }

        changed
    }

    fn fix_3b_consistency<I: Graph>(&mut self, graph: &I) {
        let nodes = graph.nodes();

        
        let mut should_continue = true;
        dbg!("This is inefficient, fix_3b_consistency");
        while should_continue {            
            should_continue = nodes.iter().map(|node| self.fix_3b_consistency_node(graph, node)).any(|x| x);
        }
    }

    // Assuming that it is 2b-consistent
    fn fix_3b_consistency_node<I: Graph>(&mut self, graph: &I, node: &I::Node) -> bool {
        let mut changed = false;  
        for j in graph.disjunctions(node) {
            for k in graph.disjunctions(node) {
                if j.id() > k.id() {
                    let tests = self.tests::<I>(node, j, k);

                    let (p_i, p_j, p_k) = (node.weight(), j.weight(), k.weight());
                        let est_i = self.constraints[node.id()].left_bound;
                        let est_j = self.constraints[j.id()].left_bound;
                        let est_k = self.constraints[k.id()].left_bound;

                        let lct_i = self.constraints[node.id()].right_bound;                        
                        let lct_j = self.constraints[j.id()].right_bound;
                        let lct_k = self.constraints[k.id()].right_bound;

                    if !tests.1 {
                        let mut next_value = lct_i;
                        // First, if jik can be satisfied...
                        let jik = (est_i <= est_j + p_j) & (est_j + p_j + p_i + p_k <= lct_k)
                            & (lct_k <= lct_i + p_k) & (lct_k - p_k < next_value);
                        let kij = (est_i <= est_k + p_k) & (est_k + p_k + p_i + p_j <= lct_j)
                            & (lct_j <= lct_i + p_j) & (lct_j - p_j < next_value);
                        
                        // This should be wrong. This takes the assumption that jik or kij will be satisfied before jki or kji
                        if jik {
                            next_value = lct_k - p_k;
                            changed = true;
                        } else if kij {
                            next_value = lct_j - p_j;
                            changed = true;
                        } else {
                            let path = std::cmp::min(est_j + p_j + p_k, est_k + p_k + p_j); // jki or kji
                            if path > next_value {
                                changed = true;
                                next_value = path;
                            }
                        }
                        self.constraints[node.id()].right_bound = next_value;
                    }

                    if !tests.0 {
                        let mut next_value = est_i;
                        // First, if jik can be satisfied...
                        if  (est_i <= est_j + p_j) & (est_j + p_j + p_i + p_k <= lct_k)
                            & (lct_k <= lct_i + p_k) & (est_j + p_j > next_value) {
                            next_value = est_j + p_j;
                            changed = true;    
                        // Else if kij can be satisfied...
                        } else if  (est_i <= est_k + p_k) & (est_k + p_k + p_i + p_j <= lct_j)
                            & (lct_j <= lct_i + p_j) & (est_k + p_k > next_value) {
                            next_value = est_j + p_j;
                            changed = true;    
                        } else {
                            let path = std::cmp::min(est_j + p_j + p_k, est_k + p_k + p_j); // jki or kji
                            if path > next_value {
                                changed = true;
                                next_value = path;
                            }
                        }
                        self.constraints[node.id()].left_bound = next_value;
                                                
                    }
                }
            }
        }

        changed
    }    
    // is it i->etc or etc->i
    fn tests<I: Graph>(&mut self, node: &I::Node, j: &I::Node, k: &I::Node) -> (bool, bool) {        
        
        let ji = self.check_precedence(j, node);
        let ki = self.check_precedence(k, node);

        let ij = self.check_precedence(node, j);
        let ik = self.check_precedence(node, k);

        let jk = self.check_precedence(j, k);
        let kj = self.check_precedence(k, j);

        ((ij & jk) | (ik & kj), (kj & ji) | (jk & ki))
    }

    pub fn check_2b_precedence<I:Graph>(&self, graph: &I) -> bool {
        graph.nodes().into_iter()
            .map(|node| graph.disjunctions(node).into_iter().map(move |o| (node, o)))
            .flatten()
            .all(|(i, j)| {
                self.check_precedence(i, j) | self.check_precedence(j, i)                
            })
    }

    pub fn check_3b_precedence<I: Graph>(&self, graph: &I) -> bool {
        graph.nodes().into_iter()
            .flat_map(|node| graph.disjunctions(node).into_iter().map(move |o| (node, o)))
            .flat_map(|(node, a)| graph.disjunctions(node).into_iter().map(move |o| (node, a, o)))
            .filter(|(_, j, k)| j.id() > k.id())
            .all(|(i, j, k)| {
                let ij = self.check_precedence(i, j);
                let ji = self.check_precedence(j, i);

                let jk = self.check_precedence(j, k);
                let kj = self.check_precedence(k, j);

                let ik = self.check_precedence(i, k);
                let ki = self.check_precedence(k, i);

                (ij & jk) | (ik & kj) |
                (ji & ik) | (jk & ki) |
                (ki & ij) | (kj & ji)
            })
    }
    
    pub fn check_precedence(&self, node_1: &impl GraphNode, node_2: &impl GraphNode) -> bool {
        // Check if arc node_1 -> node_2 is possible.
        // Meaning the earliest end date of node_1 has to be before the latest start date of node_2      

        self.constraints[node_1.id()].left_bound + node_1.weight() + node_2.weight() <= self.constraints[node_2.id()].right_bound
    }
}