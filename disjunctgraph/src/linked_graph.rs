use hashbrown::HashSet;

use crate::{ NodeId, GraphNode, ConstrainedNode, NodeIterator, Graph, Relation, GraphError, self as disjunctgraph };

#[derive(Clone, Debug)]
pub struct LinkedGraph<T: NodeId + Clone> {
    nodes: Vec<T>,
    successors: Vec<HashSet<usize>>,
    predecessors: Vec<HashSet<usize>>,
    disjunctions: Vec<HashSet<usize>>
}

impl<T: ConstrainedNode + Clone> LinkedGraph<T> {

    /// Propagate constraints for a node, note that node itself does not change, only the surrounding nodes.
    /// Propagates for the fixation that is node_1 -> node_2
    pub fn propagate(&mut self, node_1: &impl NodeId, node_2: &impl NodeId) -> Result<(), ()> {
        use std::collections::VecDeque;
        
        let mut change_occured = true;

        
        enum Propagation {
            LST { id: usize, max_lst: u32},
            EST { id: usize, min_est: u32}
        }        
        let mut stack: VecDeque<Propagation> = VecDeque::with_capacity(self.nodes.len());
        
        while change_occured {
            stack.clear();
            change_occured = false;
            // Add the successors
            let n2_est = self.nodes[node_2.id()].est() + self.nodes[node_2.id()].weight();
            for successor in self.successors(node_2) {
                stack.push_back(Propagation::EST {
                    id: successor.id(),
                    min_est: n2_est
                });
            }

            let n1_lst = self.nodes[node_1.id()].lst();
            for predecessor in self.predecessors(node_1) {
                stack.push_back(Propagation::LST {
                    id: predecessor.id(),
                    max_lst: n1_lst
                });
            }

            while let Some(prop) = stack.pop_front() {
                match prop {
                    Propagation::LST { id, max_lst } => {
                        let node = &self[id];
                        if max_lst < node.lst() {
                            if node.feasible_lst(max_lst) {
                                stack.extend(self.successors(&id).map(|s| {
                                    Propagation::LST {
                                        id: id,
                                        max_lst: max_lst + node.weight()
                                    }
                                }));
                                self[id].set_lct(max_lst);
                            } else {
                                return Err(());
                            }
                        }
                    },
                    Propagation::EST { id, min_est } => {
                        let node = &self[id];
                        if min_est > node.est() {
                            if node.feasible_est(min_est) {
                                stack.extend(self.predecessors(&id).map(|s| {
                                    Propagation::EST {
                                        id: id,
                                        min_est: min_est - s.weight()
                                    }    
                                }));
                                self[id].set_est(min_est);
                            } else {
                                return Err(());
                            }
                        }
                    }
                }
            }
            // Find all disjunctions in ineficient way for now
            for node in self.nodes().iter().map(|n| n.id()).collect::<Vec<_>>() {
                for other in self.disjunctions(&node).map(|n| n.id()).collect::<Vec<_>>() {
                    if node > other {
                        let other_lst = self.nodes[other].lst();
                        let node_ect = self.nodes[node].est() + self.nodes[node].weight();
                        // Check if node -> other is not possible 
                        if node_ect > other_lst {
                            change_occured = true;
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
                }
            }
        }

        Ok(())
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

	fn source(&self) -> &T {
		&self.nodes().first().unwrap()
	}

	fn sink(&self) -> &T {
		&self.nodes().last().unwrap()
	}

    /*fn successors<'a>(&'a self, id: &impl NodeId) -> Vec<&T> {
        self.successors[id.id()].iter().map(|x| &self.nodes[*x]).collect()
	}*/
    
    fn successors(&self, id: &impl NodeId) -> NodeIterator<Self> {
        NodeIterator(Box::new(self.successors[id.id()].iter().map(move |x| &self.nodes[*x])))
    }


    fn predecessors<'a>(&'a self, id: &impl NodeId) -> NodeIterator<Self> {
		NodeIterator(Box::new(self.predecessors[id.id()].iter().map(move |x| &self.nodes[*x])))
	}

    fn disjunctions<'a>(&'a self, id: &impl NodeId) -> NodeIterator<Self> {        
		NodeIterator(Box::new(self.disjunctions[id.id()].iter().map(move |x| &self.nodes[*x])))
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

impl<T: NodeId + Clone> std::ops::Index<usize> for LinkedGraph<T> {
    type Output = T;

   fn index(&self, node: usize) -> &Self::Output {
       &self.nodes[node]
   }
}

impl<T: NodeId + Clone> std::ops::IndexMut<usize> for LinkedGraph<T> {

   fn index_mut(&mut self, node: usize) -> &mut T {
       &mut self.nodes[node]
   }
}


impl <T: NodeId + Clone> LinkedGraph<T> {
    pub fn has_disjunctions(&self) -> bool {
        self.nodes.iter().any(|node| self.node_has_disjunction(node))        
    }

    /// Graph contains relation: node_1 -> node_2
    pub fn has_precedence(&self, node_1: &impl NodeId, node_2: &impl NodeId) -> bool {
        self.successors[node_1.id()].contains(&node_2.id())
    }

    pub fn has_disjunction(&self, node_1: &impl NodeId, node_2: &impl NodeId) -> bool {
        self.disjunctions[node_1.id()].contains(&node_2.id())
    }

    pub fn node_has_disjunction(&self, node: &impl NodeId) -> bool {
        !self.disjunctions[node.id()].is_empty()
    }
}