#[derive(Clone, Debug)]
pub struct Node {
    id: usize,
    job_id: Option<usize>,
    weight: u32,    
    machine_id: Option<u32>,
    head: Option<u32>,
    tail: Option<u32>
}

impl disjunctgraph::NodeId for Node {
    fn id(&self) -> usize { self.id }
}

impl disjunctgraph::GraphNode for Node {
    fn create(id: usize, weight: u32, machine_id: Option<u32>, job_id: Option<usize>) -> Self {
        Node {
            id, weight, job_id, machine_id,
            head: None, 
            tail: None
        }
    }
    fn weight(&self) -> u32 { self.weight }
    fn job_id(&self) -> Option<usize> { self.job_id }
    fn machine_id(&self) -> Option<u32> {self.machine_id }
}

impl disjunctgraph::ConstrainedNode for Node {
    fn head(&self) -> u32 {
        self.head.expect("Head not initialized")
    }

    fn tail(&self) -> u32 {
        self.tail.expect("Tail not initialized")
    }
    fn set_head(&mut self, head: u32) {
        self.head = Some(head);
    }
    fn set_tail(&mut self, tail: u32) {
        self.tail = Some(tail);
    }
}