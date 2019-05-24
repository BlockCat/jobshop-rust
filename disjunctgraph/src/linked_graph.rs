use hashbrown::HashSet;

use crate::{ NodeId, GraphNode, ConstrainedNode, NodeIterator, Graph, Relation, GraphError, self as disjunctgraph };

#[derive(Clone, Debug)]
pub struct LinkedGraph<T: NodeId + Clone> {
    nodes: Vec<T>,
    successors: Vec<HashSet<usize>>,
    predecessors: Vec<HashSet<usize>>,
    disjunctions: Vec<HashSet<usize>>
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
    
    fn successors(&self, id: &impl NodeId) -> NodeIterator<Self> {
        NodeIterator(Box::new(self.successors[id.id()].iter().map(move |x| &self.nodes[*x])))
    }


    fn predecessors<'a>(&'a self, id: &impl NodeId) -> NodeIterator<Self> {
		NodeIterator(Box::new(self.predecessors[id.id()].iter().map(move |x| &self.nodes[*x])))
	}

    fn disjunctions<'a>(&'a self, id: &impl NodeId) -> NodeIterator<Self> {        
		NodeIterator(Box::new(self.disjunctions[id.id()].iter().map(move |x| &self.nodes[*x])))
	}

    fn fix_disjunction(&mut self, node_1: &impl NodeId, node_2: &impl NodeId) -> Result<(), GraphError> {
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

        Ok(())       
		
        /*if self.is_cyclic() {
            Err(disjunctgraph::GraphError::Cyclic)
        } else {
            Ok(())
        }*/
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

    /// Graph contains relation: node_1 -> node_2
    fn has_precedence(&self, node_1: &impl NodeId, node_2: &impl NodeId) -> bool {
        self.successors[node_1.id()].contains(&node_2.id())
    }

    fn has_disjunction(&self, node_1: &impl NodeId, node_2: &impl NodeId) -> bool {
        self.disjunctions[node_1.id()].contains(&node_2.id())
    }

    fn node_has_disjunction(&self, node: &impl NodeId) -> bool {
        !self.disjunctions[node.id()].is_empty()
    }

    fn search_orders(&mut self, upper_bound: u32) -> bool where Self::Node: ConstrainedNode {
        
        let mut change_occured = false;
        for node in self.nodes().iter().map(|n| n.id()).collect::<Vec<_>>() {
            for other in self.disjunctions(&node).map(|n| n.id()).collect::<Vec<_>>() {

                    let head = self[node].head();
                    let processing = self[node].weight() + self[other].weight();
                    
                    let tail = self[other].tail();
                    
                    // Check if node -> other is not possible 
                    // node: 1 -> other: 3
                    // node_ect: 2
                    // other_lst: 6
                    // If node -> other is bigger than the upper bound
                    if head + processing + tail > upper_bound {
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

        change_occured
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
    
}

impl<T: ConstrainedNode + Clone> std::fmt::Display for LinkedGraph<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "successors:\n").unwrap();
        for node in self.nodes() {
            write!(f, "{} -> {:?}\n", node.id(), self.successors(&node.id()).map(|x| x.id()).collect::<Vec<_>>()).unwrap();
        }

        write!(f, "predecessor:\n").unwrap();
        for node in self.nodes() {
            write!(f, "{} -> {:?}\n", node.id(), self.predecessors(&node.id()).map(|x| x.id()).collect::<Vec<_>>()).unwrap();
        }

        write!(f, "disjunction:\n").unwrap();
        for node in self.nodes() {
            write!(f, "{} -> {:?}\n", node.id(), self.disjunctions(&node.id()).map(|x| x.id()).collect::<Vec<_>>()).unwrap();
        }

        write!(f, "")
    }
}