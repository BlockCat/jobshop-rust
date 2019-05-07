#[derive(Clone, Debug)]
pub struct Node {
    id: usize,
    job_id: Option<usize>,
    weight: u32,    
    machine_id: Option<u32>,
    est: Option<u32>,
    lst: Option<u32>
}

impl disjunctgraph::NodeId for Node {
    fn id(&self) -> usize { self.id }
}

impl disjunctgraph::GraphNode for Node {
    fn create(id: usize, weight: u32, machine_id: Option<u32>, job_id: Option<usize>) -> Self {
        Node {
            id, weight, job_id, machine_id,
            est: None, 
            lst: None
        }
    }
    fn weight(&self) -> u32 { self.weight }
    fn job_id(&self) -> Option<usize> { self.job_id }
    fn machine_id(&self) -> Option<u32> {self.machine_id }
}

impl Node {
    pub fn earliest_start_time(&self) -> Option<u32> {
        self.est
    }
}