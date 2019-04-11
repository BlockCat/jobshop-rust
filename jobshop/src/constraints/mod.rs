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
        
        let mut should_continue = true;
        dbg!("This is inefficient, fix_2b_consistency");
        while should_continue {            
            should_continue = nodes.iter().map(|node| self.fix_2b_consistency_node(graph, node)).any(|x| x);
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
            if (est_i + p_i > lst_j) && (est_j + p_j > est_i) { // Executes 2-b-consistency rule
                self.constraints[node.id()].left_bound = est_j + p_j;
                changed = true;
            }

            if (est_j + p_j > lst_i) && (lst_j < lst_i + p_i) {
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

                    if !tests {
                        let (p_i, p_j, p_k) = (node.weight() as i32, j.weight() as i32, k.weight() as i32);
                        let est_i = self.constraints[node.id()].left_bound as i32;
                        let est_j = self.constraints[j.id()].left_bound as i32;
                        let est_k = self.constraints[k.id()].left_bound as i32;
                        
                        let lct_i = self.constraints[node.id()].right_bound as i32;
                        let lct_j = self.constraints[j.id()].right_bound as i32;
                        let lct_k = self.constraints[k.id()].right_bound as i32;

                        // Est_i consistency tests
                        let next_value = std::cmp::min(est_j, est_k) + p_j + p_k;
                        let consistency_test_1 = {
                            let m = *([
                                lct_j - est_i, 
                                lct_j - est_k, 
                                lct_k - est_i, 
                                lct_k - est_j]
                                .into_iter().max().unwrap());
                            m < p_i + p_j + p_k && next_value > est_i
                        };

                        let consistency_test_2 = est_i + p_i > std::cmp::max(lct_j - p_j, lct_k - p_k) && next_value > est_i;
                        let consistency_test_3 = std::cmp::max(lct_j - est_i, lct_k - est_i) < p_i + p_j + p_k 
                            && std::cmp::min(est_j + p_j, est_k + p_k) > est_i;
                        
                        if consistency_test_1 || consistency_test_2 {
                            self.constraints[node.id()].left_bound = next_value as u32;
                            changed = true;
                        } 

                        if consistency_test_3 {
                            self.constraints[node.id()].left_bound = std::cmp::min(est_j + p_j, est_k + p_k) as u32;
                            changed = true;
                        }

                        // Lst_i consistency tests
                        let m = *([
                            lct_i - est_j, 
                            lct_i - est_k, 
                            lct_j - est_k, 
                            lct_k - est_j]
                            .into_iter()
                            .max().unwrap());
                        
                        let next_value = std::cmp::max(lct_j, lct_k) - p_j - p_k;

                        if (m < p_i + p_j + p_k) & (next_value < lct_i) {
                            self.constraints[node.id()].right_bound = next_value as u32;
                            changed = true;
                        }
                        if (std::cmp::min(est_j + p_j, est_k + p_k) + p_i > lct_i) & (next_value < lct_i) {
                            self.constraints[node.id()].right_bound = next_value as u32;
                            changed = true;
                        }
                        let next_value = std::cmp::max(lct_j - p_j, lct_k - p_k);
                        let consistency_test_6 = std::cmp::max(lct_i - est_j, lct_i - est_k) < p_i + p_j + p_k
                            && next_value < lct_i;

                        if  consistency_test_6 {
                            self.constraints[node.id()].right_bound = next_value as u32;
                        }
                    }
                }
            }
        }

        changed
    }    

    fn tests<I: Graph>(&mut self, node: &I::Node, j: &I::Node, k: &I::Node) -> bool {
        let (p_i, p_j, p_k) = (node.weight(), j.weight(), k.weight());
        let est_i = self.constraints[node.id()].left_bound;

        let lct_j = self.constraints[j.id()].right_bound;
        let lct_k = self.constraints[k.id()].right_bound;

        std::cmp::max(lct_j, lct_k) >= p_i + p_j + p_k + est_i
    }

    pub fn check_2b_precedence<I:Graph>(&self, graph: &I) -> bool {
        graph.nodes().into_iter()
            .map(|node| graph.disjunctions(node).into_iter().map(move |o| (node, o)))
            .flatten()
            .all(|(node, disj)| {
                let est_i = self.constraints[node.id()].left_bound;
                let lct_i = self.constraints[node.id()].right_bound;
                let p_i = node.weight();

                let est_j = self.constraints[disj.id()].left_bound;
                let lct_j = self.constraints[disj.id()].right_bound;                
                let p_j = disj.weight();

                est_i + p_i + p_j <= lct_j || est_j + p_j + p_i <= lct_i
            })
    }

    pub fn check_3b_precedence<I: Graph>(&self, graph: &I) -> bool {
        graph.nodes().into_iter()
            .flat_map(|node| graph.disjunctions(node).into_iter().map(move |o| (node, o)))
            .flat_map(|(node, a)| graph.disjunctions(node).into_iter().map(move |o| (node, a, o)))
            .filter(|(_, j, k)| j.id() > k.id())
            .all(|(i, j, k)| {
                let est_i = self.constraints[i.id()].left_bound;
                let est_j = self.constraints[j.id()].left_bound;
                let est_k = self.constraints[k.id()].left_bound;
                
                let lct_i = self.constraints[i.id()].right_bound;
                let lct_j = self.constraints[j.id()].right_bound;
                let lct_k = self.constraints[k.id()].right_bound;                

                let p_i = i.weight();
                let p_j = j.weight();
                let p_k = k.weight();

                (est_i + p_i <= est_j && est_j + p_j + p_k <= lct_k)
                || (est_i + p_i <= est_k && est_k + p_k + p_j <= lct_j)
                || (est_j + p_j <= est_i && est_i + p_i + p_k <= lct_k)
                || (est_k + p_k <= est_i && est_i + p_i + p_j <= lct_j)
                || (est_j + p_j <= est_k && est_k + p_k <= est_i)
                || (est_k + p_k <= est_j && est_j + p_j <= est_i)
            })
    }
    
    pub fn check_precedence(&self, node_1: &impl GraphNode, node_2: &impl GraphNode) -> bool {
        // Check if arc node_1 -> node_2 is possible.
        // Meaning the earliest end date of node_1 has to be before the latest start date of node_2      

        self.constraints[node_1.id()].left_bound + node_1.weight() + node_2.weight() <= self.constraints[node_2.id()].right_bound
    }

    pub fn set_upper_bound(&mut self, _upper_bound: u32) -> Result<(), ConstraintError> {
        unimplemented!()
    }
}