mod linked_graph;

use std::collections::VecDeque;

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

const TOPOLOGY_PROCESSED: u8 = 1;
const TOPOLOGY_IN_STACK: u8 = 2;
enum Status { Visisted(usize), Unvisited(usize) }
pub struct TopologyIterator<'a, G: Graph> {
    graph: &'a G,
    node_state: Vec<u8>, // processed, in_stack as bitflags
    stack: VecDeque<Status>
}

impl<'a, G: Graph> Iterator for TopologyIterator<'a, G> {
    type Item = &'a G::Node;

    fn next(&mut self) -> Option<Self::Item> {
        loop { // Loop till it exits

            // Pop node from stack
            let current_node = match self.stack.pop_back() {
                Some(node) => node,
                _ => return None, // If no node in the stack, then topology exits
            };

            match current_node {
                // If the node is unvisited expand it.
                Status::Unvisited(current_node) => {
                    if self.node_state[current_node] == 0 { // It's not in stack and not processed
                        self.stack.push_back(Status::Visisted(current_node));                        
                        self.stack.extend(self.graph.successors(&current_node).iter().map(|x| Status::Unvisited(x.id())));                        
                        self.node_state[current_node] |= TOPOLOGY_IN_STACK; // It's now in stack
                    }
                },
                // If the node is visited and not yet processed, return it.
                Status::Visisted(current_node) => {
                    if (self.node_state[current_node] & TOPOLOGY_PROCESSED) == 0 {
                        // Set node to be not in stack but yes processed
                        self.node_state[current_node] = TOPOLOGY_PROCESSED;
                        dbg!(current_node);
                        return Some(&self.graph.nodes()[current_node]);
                    }
                }
            }
        }
    }
}

pub trait Graph where Self: Sized {
    type Node: NodeId + GraphNode;
    fn create(nodes: Vec<Self::Node>, edges: Vec<Vec<Relation>>) -> Self;
    fn nodes(&self) -> &[Self::Node];
    fn source(&self) -> &Self::Node;
    fn sink(&self) -> &Self::Node;
    fn successors(&self, id: &impl NodeId) -> Vec<&Self::Node>;
    fn predecessors(&self, id: &impl NodeId) -> Vec<&Self::Node>;
    fn disjunctions(&self, id: &impl NodeId) -> Vec<&Self::Node>;
    fn fix_disjunction(mut self, node_1: &impl NodeId, node_2: &impl NodeId) -> Result<Self, GraphError>;    
    fn flip_edge(mut self, node_1: &impl NodeId, node_2: &impl NodeId) -> Result<Self, GraphError>;
    fn into_directed(&self) -> Result<Self, GraphError>;

    fn topology1<'a>(&'a self) -> TopologyIterator<'a, Self> {

        let mut stack = VecDeque::with_capacity(self.nodes().len());
        stack.push_back(Status::Unvisited(self.source().id()));

        TopologyIterator {
            graph: self,
            node_state: vec!(0u8; self.nodes().len()),            
            stack: stack
        }
    }
    /*
    fn topology(&self) -> Vec<u16> {
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

        let mut stack: VecDeque<Status> = VecDeque::with_capacity(processed.len());

        stack.push_back(Status::Unvisited(source.id()));
        let mut counter = 1;

        while !stack.is_empty() {            
            let current_node = stack.pop_back().unwrap();

            match current_node {
                Status::Unvisited(current_node) => {
                    if processed[current_node] == 0 && !in_stack[current_node] {                        
                        stack.push_back(Status::Visisted(current_node));                        
                        stack.extend(self.successors(&current_node).iter().map(|x| Status::Unvisited(x.id())));
                        in_stack[current_node] = true;
                    }
                },
                Status::Visisted(current_node) => {
                    if processed[current_node] == 0 {
                        processed[current_node] = counter;
                        in_stack[current_node] = false;

                        counter += 1;
                    }
                }
            }
        }
        processed
    }*/

    fn critical_length(&self) -> std::result::Result<u32, GraphError> {
        
        let mut starting_times = vec!(0u32; self.nodes().len());
        
        for node in self.topology1() {
            let predecessors = self.predecessors(node);

            let nodes = self.nodes();            
            starting_times[node.id()] = predecessors.iter()                
                .map(|x| starting_times[x.id()] + nodes[x.id()].weight())
                .max().unwrap_or(0);
        }

        Ok(*starting_times.last().unwrap())
    }

    fn critical_path(&self) -> std::result::Result<(u32, Vec<&Self::Node>), GraphError> {
        
        
        // Starting with the node with the highest topology, the source...
        let mut starting_times = vec!(0u32; self.nodes().len());
        let mut backtracker = vec!(0usize; self.nodes().len());

        for node in self.topology1() {            
            
            /*let any_predecessor_smaller = predecessors.iter().any(|x| topology[x.id()] < topology[node] );                        
            if any_predecessor_smaller {
                return Err(GraphError::Cyclic);
            }*/

            let nodes = self.nodes();   
            let max_predecessor = self
                .predecessors(node).iter()
                .map(|x| (x.id(), starting_times[x.id()] + nodes[x.id()].weight()))                
                .max_by_key(|x| x.1);

            if let Some(max_predecessor) = max_predecessor {                 
                backtracker[node.id()] = max_predecessor.0;
                starting_times[node.id()] = max_predecessor.1;
            }
        }

        let max_span = starting_times.last().unwrap();
        let mut path = Vec::new();
        let mut pointer = backtracker[self.sink().id()];
        while pointer != 0 {
            let prev = backtracker[pointer];
            let node = &self.nodes()[pointer];
            path.push(node);
            pointer = prev;
        }
        path.reverse();
        Ok((*max_span, path))
    }

    fn is_cyclic(&self) -> bool {
        
        // Start DFS from source        
        let mut processed = vec!(0u16; self.nodes().len());
        let mut counter = 0;

        for node in self.topology1() {
            counter += 1;
            processed[node.id()] = counter;

            // Because it is in a topologic order:
            // sink would have the lowest value,
            // source would have the largest value,
            // therefore, all predecessors should have larger value than current node.
            let is_cyclic = self.predecessors(node).iter()
                .any(|x| processed[x.id()] > counter);
                        
            if is_cyclic {
                return true;
            }
        }
        false
    }
}