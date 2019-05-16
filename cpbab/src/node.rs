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
    
    fn est(&self) -> u32 {
        self.est.expect("Could not get earliest starting time!")
    }

    fn set_lct(&mut self, lct: u32) {
        debug_assert!(lct >= self.weight);
        debug_assert!(match self.est {
            Some(est) => lct - self.weight >= est,
            None => true
        }, "wrong lst: {} for {:?}", lct - self.weight, self);
        self.lst = Some(lct - self.weight);
    }
    
    fn lct(&self) -> u32 {
        self.lst.expect("Could not get latest completion time") + self.weight
    }
}