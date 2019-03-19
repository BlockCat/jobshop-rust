mod linked_graph;

pub use linked_graph::LinkedGraph;

#[derive(Debug)]
pub enum GraphError {
    Cyclic, InvalidEdge
}

pub trait NodeId {
    fn id(&self) -> usize;
}

pub trait GraphNode: NodeId {
    fn create(id: usize, weight: u32, machine_id: Option<u32>, job_id: Option<usize>) -> Self;
    fn weight(&self) -> u32;
    fn job_id(&self) -> Option<usize>;
    fn machine_id(&self) -> Option<u32>;
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

pub trait Graph<T> where T: NodeId + GraphNode, Self: Sized {
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

    fn critical_path(&self) -> std::result::Result<(u32, Vec<&T>), GraphError> {        
        if self.is_cyclic() {
            Err(GraphError::Cyclic)
        } else {
            Ok(self.force_critical_path())
        }
    }

    fn force_critical_path(&self) -> (u32, Vec<&T>) {
        use std::collections::VecDeque;
        // Use shortest path algorithm (modified)
        // with negative edges
        let source = self.source().id();

        let mut processed = vec!(0u32; self.nodes().len());
        let mut backtracker = vec!(0usize; self.nodes().len());
        let mut stack: VecDeque<usize> = VecDeque::new();
        //stack.push_back(source);
        stack.extend(self.successors(&source).iter().map(|x| x.id()));

        while !stack.is_empty() {
            let current_node = stack.pop_front().unwrap();
            let weight = self.nodes()[current_node].weight();            

            for successor in self.successors(&current_node) {
                let successor = successor.id();                
                if processed[successor] <= processed[current_node] + weight {
                    processed[successor] = processed[current_node] + weight;
                    backtracker[successor] = current_node;
                    stack.push_back(successor);
                }
            }
        }

        let mut path = Vec::new();
        let mut pointer = backtracker[self.sink().id()];
        while pointer != 0 {
            let prev = backtracker[pointer];
            let node = &self.nodes()[pointer];
            path.push(node);
            pointer = prev;
        }
        path.reverse();
        //println!("Backtrack: {:?}", path.iter().map(|x| x.id()).collect::<Vec<_>>());
        //println!("Criticals: {:?}", processed.last().unwrap());
        
        (*processed.last().unwrap(), path)
    }

    fn is_cyclic(&self) -> bool {
        use std::collections::VecDeque;
        // Start DFS from source
        let source = self.source();

        let mut processed = vec!(0u16; self.nodes().len());
        let mut in_stack = vec!(false; self.nodes().len());

        // Create a topology ordering in O(V + E)
        // postorder
        // Without recursion:
        // Add the node in the stack as visited, then add children.
        // Once children are done the node will be visited again postorder action is taken.
        // To prevent loops, the node will only be handled if it doesn't have a topology order and it's not in the stack.

        enum Status { Visisted(usize), Unvisited(usize) };
        let mut stack: VecDeque<Status> = VecDeque::with_capacity(processed.len() / 4);

        stack.push_back(Status::Unvisited(source.id()));
        let mut counter = 1;

        while !stack.is_empty() {            
            let current_node = stack.pop_back().unwrap();

            match current_node {
                Status::Unvisited(current_node) => {
                    if processed[current_node] == 0 && !in_stack[current_node] {
                        // I should go after the children, so add me first.
                        stack.push_back(Status::Visisted(current_node));
                        // Then add children, because stack
                        stack.extend(self.successors(&current_node).iter().map(|x| Status::Unvisited(x.id())));

                        in_stack[current_node] = true;
                    }
                },
                Status::Visisted(current_node) => {
                    if processed[current_node] == 0 {
                        processed[current_node] = counter;                    
                        in_stack[current_node] = false;                        

                        // Find contradiction
                        let any_predecessor_smaller = self.predecessors(&current_node).iter()
                                .any(|x| processed[x.id()] > 0 && processed[x.id()] < counter );
                        
                        if any_predecessor_smaller {
                            return true;
                        }

                        counter += 1;
                    }
                }
            }
        }
        false
    }
}