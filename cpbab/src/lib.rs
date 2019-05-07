extern crate disjunctgraph;

mod node;

// Constrained graph ;)
type CGraph = disjunctgraph::LinkedGraph<node::Node>;

const PAR: u32 = 3;


pub fn branch_and_bound() {
    loop {

    }
}

struct TaskInterval<'a> {
    upper: u32,
    lower: u32,
    processing: u32,
    nc: NCData<'a>,
    nodes: Vec<&'a node::Node>,
}

struct NCData<'a> {
    start: bool,
    nodes: Vec<&'a node::Node>
}

impl<'a> TaskInterval<'a> {
    fn slack(&self) -> u32 {
        self.upper - self.lower - self.processing
    }
}

fn ff(resource_id: usize, graph: &CGraph) -> u32 {
    let mut tasks_on_resource: Vec<&node::Node> = unimplemented!();

    tasks_on_resource.sort_unstable_by_key(|x| {
        x.earliest_start_time()
    });
    
    let crit = crit(resource_id, graph);
    let resource_slack: TaskInterval = unimplemented!();
    

    crit.slack() * resource_slack.slack() * std::cmp::min(PAR, crit.nc.nodes.len() as u32)
}

fn crit<'a>(resource_id: usize, graph: &'a CGraph) -> TaskInterval<'a> {

    

    let task_intervals: Vec<TaskInterval<'a>> = unimplemented!();

    task_intervals.into_iter()
        .map(NC)
        .min_by_key(|x| x.slack() * x.nc.nodes.len() as u32)
        .unwrap()
}

fn NC(task_interval: TaskInterval) -> TaskInterval {
    let possible_first: Vec<&node::Node> = unimplemented!();
    let possible_last: Vec<&node::Node> = unimplemented!();

    task_interval.nc = if possible_first.len() < possible_last.len() {
        NCData {
            start: true,
            nodes: possible_first
        }
    } else {
        NCData {
            start: false,
            nodes: possible_last
        }
    };

    task_interval
}



fn lower_bound() -> u32 {
    unimplemented!()
}