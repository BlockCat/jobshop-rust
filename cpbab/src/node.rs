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

impl disjunctgraph::ConstrainedNode for Node {
    fn set_est(&mut self, est: u32) {
        debug_assert!(match self.lst {
                Some(lst) => lst >= est,
                None => true
        });
        self.est = Some(est);
    }

    fn set_lst(&mut self, lst: u32) {
        self.lst = Some(lst);
    }
    
    fn est(&self) -> u32 {
        self.est.expect("Could not get earliest starting time!")
    }

    fn lst(&self) -> u32 {
        self.lst.expect("Could not get latest starting time for node")
    }    
}