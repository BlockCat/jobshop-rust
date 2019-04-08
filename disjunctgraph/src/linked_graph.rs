use std::collections::HashSet;

use crate::{ NodeId, GraphNode, Graph, Relation, GraphError, self as disjunctgraph };

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
        		
        if self.is_cyclic() {
            Err(disjunctgraph::GraphError::Cyclic)
        } else {
            Ok(self)
        }
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