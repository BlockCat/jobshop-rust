use std::collections::HashMap;

mod matrix_graph;
mod linked_graph;

pub use linked_graph::LinkedGraph;

#[derive(Debug)]
pub enum GraphError {
    Cyclic
}

pub trait NodeId {
    fn id(&self) -> usize;
}

pub trait NodeWeight {
    fn weight(&self) -> u32 {
        1
    }
}

impl NodeId for usize {
    fn id(&self) -> usize {
        *self
    }
}

#[derive(Clone, PartialEq)]
pub enum Relation {
    Successor(usize), Predecessor(usize), Disjunctive(usize)
}

pub trait Graph<T> where T: NodeId + NodeWeight, Self: Sized {
    fn create(nodes: Vec<T>, edges: Vec<Vec<Relation>>) -> Self;
    fn nodes(&self) -> &[T];
    fn source(&self) -> &T;
    fn sink(&self) -> &T;
    fn successors(&self, id: &impl NodeId) -> Vec<&T>;
    fn predecessors(&self, id: &impl NodeId) -> Vec<&T>;
    fn disjunctions(&self, id: &impl NodeId) -> Vec<&T>;
    fn fix_disjunction(&self, node_1: &impl NodeId, node_2: &impl NodeId) -> Result<Self, GraphError>;
    fn flip_edge(&self, node_1: &impl NodeId, node_2: &impl NodeId) -> Result<Self, GraphError>;
    fn into_directed(&self) -> Result<Self, GraphError>;

    fn critical_path(&self) -> std::result::Result<Vec<&T>, GraphError> {        
        if self.is_cyclic() {
            Err(GraphError::Cyclic)
        } else {
            Ok(self.force_critical_path())
        }
    }

    fn force_critical_path(&self) -> Vec<&T> {
        // Use dijkstras algorithm (modified)

        unimplemented!()
    }

    fn is_cyclic(&self) -> bool {
        use std::collections::VecDeque;
        // Start DFS from source
        let source = self.source();

        let mut processed = vec!(0u16; self.nodes().len());

        // Create a topology ordering in O(V + E)
        let mut stack: VecDeque<usize> = VecDeque::with_capacity(processed.len() / 4);
        stack.push_back(source.id());
        let mut counter = 0;
        while !stack.is_empty() {            
            let current_node = stack.pop_front().unwrap();
            if processed[current_node] == 0 {
                counter += 1;
                processed[current_node] = counter;           

                // Add children
                stack.extend(self.successors(&current_node).iter().map(|x| x.id()));   
            }
        }

        //*processed.last_mut().unwrap() = std::u16::MAX;
        
        println!("topology ordering: {:?}", processed);        
        
        // Find contradiction in O(N + M)
        for node in self.nodes() {
            let node = node.id();
            for successor in self.successors(&node) {
                let successor = successor.id();

                if processed[successor] <= processed[node] {
                    return true;
                }
            }
        }

        false
    }
}