use hashbrown::HashSet;

use crate::{ NodeId, GraphNode, ConstrainedNode, Graph, Relation, GraphError, self as disjunctgraph };

#[derive(Clone, Debug)]
pub struct LinkedGraph<T: NodeId + Clone> {
    nodes: Vec<T>,
    successors: Vec<HashSet<usize>>,
    predecessors: Vec<HashSet<usize>>,
    disjunctions: Vec<HashSet<usize>>
}

impl<T: ConstrainedNode + Clone> LinkedGraph<T> {
    pub fn init_weights(&mut self, max_makespan: u32) {        
        let topology = self.topology().map(|x| x.id()).collect::<Vec<_>>();

        for node in &topology {
            let head = self.predecessors(node).into_iter().map(|x| x.est() + x.weight()).max().unwrap_or(0);
            self.nodes[*node].set_est(head);
        }

        for node in topology.iter().rev() {
            let tail = self.successors(node).into_iter().map(|x| x.lct() - x.weight()).min().unwrap_or(max_makespan);
            self.nodes[*node].set_lct(tail);
        }
    }

    /// Propagate constraints for a node, note that node itself does not changed, only the surrounding nodes.
    pub fn propagate(&mut self, node: &impl NodeId) {        
        
        let node_est = self.nodes[node.id()].est();
        let node_lct = self.nodes[node.id()].lct();
        let node_weight = self.nodes[node.id()].weight();

        for other in self.disjunctions(node).iter().map(|x| x.id()).collect::<Vec<_>>() {
            let other_lst = self.nodes[other].lct() - self.nodes[other].weight();
            let node_ect = node_est + node_weight;
            // Check if node -> other is not possible 
            if node_ect > other_lst {
                // other -> node it is then
                let node_1 = other;
                let node_2 = node.id();
                self.disjunctions[node_1].remove(&node_2);
                self.disjunctions[node_2].remove(&node_1);
                
                // Node_1 -> Node_2
                self.successors[node_1].insert(node_2);
                self.predecessors[node_2].insert(node_1);     
            }
        }
        
        // For the predecessors, check if their lct needs to be changed.
        for other in self.predecessors(node).iter().map(|x| x.id()).collect::<Vec<_>>() {
            let other_lct = self.nodes[other].lct();
            if node_lct - node_weight < other_lct {
                self.nodes[other].set_lct(node_lct - node_weight);
                self.propagate(&other);
            }
        }

        for other in self.successors(node).iter().map(|x| x.id()).collect::<Vec<_>>() {
            let other_est = self.nodes[other].est();
            if node_est + node_weight > other_est {
                self.nodes[other].set_est(node_est + node_weight);
                self.propagate(&other);
            }
        }
    }
}

impl<T: NodeId + GraphNode + Clone> Graph for LinkedGraph<T> {
    type Node = T;

    fn create(nodes: Vec<T>, edges: Vec<Vec<Relation>>) -> Self {

        let successors = edges.iter()
            .map(|x| x.iter().filter_map(|x| {
                    match x {
                        Relation::Successor(e) => Some(*e),
                        _ => None
                    }
            }).collect::<HashSet<_>>());

        let predecessors = edges.iter()
            .map(|x| x.iter().filter_map(|x| {
                    match x {
                        Relation::Predecessor(e) => Some(*e),
                        _ => None
                    }
            }).collect::<HashSet<_>>());

        let disjunctions = edges.iter()
            .map(|x| x.iter().filter_map(|x| {
                    match x {
                        Relation::Disjunctive(e) => Some(*e),
                        _ => None
                    }
            }).collect::<HashSet<_>>());
        
        LinkedGraph {
            nodes,
            successors: successors.collect(),
            predecessors: predecessors.collect(),
            disjunctions: disjunctions.collect()
        }
    }

    fn nodes(&self) -> &[T] {
		&self.nodes
	}

    fn nodes_mut(&mut self) -> &mut [T] {
        &mut self.nodes
    }

	fn source(&self) -> &T {
		&self.nodes().first().unwrap()
	}

	fn sink(&self) -> &T {
		&self.nodes().last().unwrap()
	}

    fn successors<'a>(&'a self, id: &impl NodeId) -> Vec<&T> {
        self.successors[id.id()].iter().map(|x| &self.nodes[*x]).collect()
	}

    fn predecessors<'a>(&'a self, id: &impl NodeId) -> Vec<&T> {
		self.predecessors[id.id()].iter().map(|x| &self.nodes[*x]).collect()
	}

    fn disjunctions<'a>(&'a self, id: &impl NodeId) -> Vec<&T> {
		self.disjunctions[id.id()].iter().map(|x| &self.nodes[*x]).collect()
	}

    fn fix_disjunction(mut self, node_1: &impl NodeId, node_2: &impl NodeId) -> Result<Self, GraphError> {
        if !self.disjunctions[node_1.id()].contains(&node_2.id()) {
            return Err(GraphError::InvalidEdge);
        }

        let node_1 = node_1.id();
        let node_2 = node_2.id();
        
        // Remove from disjunctions        
        self.disjunctions[node_1].remove(&node_2);
        self.disjunctions[node_2].remove(&node_1);
        
        // Node_1 -> Node_2
        self.successors[node_1].insert(node_2);
        self.predecessors[node_2].insert(node_1);        
		
        if self.is_cyclic() {
            Err(disjunctgraph::GraphError::Cyclic)
        } else {
            Ok(self)
        }
	}

    fn flip_edge(mut self, node_1: &impl NodeId, node_2: &impl NodeId) -> Result<Self, GraphError> {
        if !self.successors[node_1.id()].contains(&node_2.id()) {
            return Err(GraphError::InvalidEdge);
        }        
        
        let node_1 = node_1.id();
        let node_2 = node_2.id();
        // node_1 -> node_2
        self.successors[node_1].remove(&node_2);
        self.predecessors[node_2].remove(&node_1);

        self.predecessors[node_1].insert(node_2);
        self.successors[node_2].insert(node_1);
        		
        // if self.is_cyclic() {
        //     Err(disjunctgraph::GraphError::Cyclic)
        // } else {
        //     Ok(self)
        // }
        Ok(self)
	}

    // For now, the better way will probably be to create a topological ordering and go from there
    fn into_directed(&self) -> Result<Self, GraphError> {
        let mut cloned = self.clone();
        // For every disjunction, flip edge
		for node_1 in 0..(self.nodes.len()-1) {
            for node_2 in self.disjunctions[node_1].iter().cloned() {
                if node_2 > node_1 {
                    // It'll become node_1 -> node_2
                    cloned.disjunctions[node_1].remove(&node_2);
                    cloned.disjunctions[node_2].remove(&node_1);

                    cloned.successors[node_1].insert(node_2);
                    cloned.predecessors[node_2].insert(node_1);
                }
            }
        }

        if cloned.is_cyclic() {
            Err(disjunctgraph::GraphError::Cyclic)
        } else {
            Ok(cloned)
        }
	}
}

impl <T: NodeId + Clone> LinkedGraph<T> {
    pub fn has_disjunctions(&self) -> bool {
        self.nodes.iter().any(|node| !self.disjunctions[node.id()].is_empty())
    }

    pub fn has_disjunction(&self, node_1: &impl NodeId, node_2: &impl NodeId) -> bool {
        self.disjunctions[node_1.id()].contains(&node_2.id())
    }

    pub fn node_has_disjunction(&self, node: &impl NodeId) -> bool {
        !self.disjunctions[node.id()].is_empty()
    }
}