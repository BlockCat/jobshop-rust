use std::io::{ BufRead, BufReader };
use std::fs::File;
use std::path::Path;

use crate::schedule::Schedule;

use disjunctgraph::{ Graph, NodeId, GraphNode, Relation };


pub trait ProblemSolver {
    type Solution;
    fn solve(&self, problem: &Problem) -> Self::Solution;
}

#[derive(Debug, Clone)]
pub struct Problem {
    pub machines: u32,
    pub activities: Vec<Activity>,
    pub jobs: Vec<Vec<usize>>,
    pub optimal: u32,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Activity {
    pub id: usize,
    pub process_time: u32,
    pub machine_id: u32,
    pub precedences: Vec<usize>,    
}

// To be ran with: https://www.eii.uva.es/elena/JSSP/InstancesJSSP.htm
impl Problem {
    pub fn read<P: AsRef<Path>>(path: P) -> Result<Self, String> {
        let file = File::open(path).map_err(|_| "Could not read file".to_owned())?;
        let reader = BufReader::new(file);

        let mut reader = reader.lines().map(|x| x.unwrap());
        
        let jobs = reader.next().and_then(|x| x.parse::<u32>().ok()).ok_or("Machines is not a number")?;
        let machines = reader.next().and_then(|x| x.parse::<u32>().ok()).ok_or("Jobs is not a number")?;
        let optimal = reader.next().and_then(|x| x.parse::<u32>().ok()).ok_or("No optimal given")?;

        // Read activity processing times
        let processing_times = reader.by_ref()
            .take(jobs as usize)
            .map(|x| x.split(" ").map(|s| s.parse::<u32>().unwrap()).collect::<Vec<_>>())
            .collect::<Vec<_>>().into_iter();

        // Read activity machine placement
        let machine_placements = reader.by_ref()
            .take(jobs as usize)
            .map(|x| x.split(" ").map(|s| s.parse::<u32>().unwrap()).collect::<Vec<_>>())
            .collect::<Vec<_>>().into_iter();

        // Merge processing times and machine placements
        let mut counter = 0usize;
        let mut activities: Vec<Vec<Activity>> = processing_times.zip(machine_placements).map(|(p, m)| {
                p.into_iter().zip(m).map(|(p, m)| {
                    let id = counter;
                    counter += 1;
                    Activity {
                        id: id,
                        process_time: p,
                        machine_id: m,
                        precedences: Default::default()
                    }
                }).collect()
            }).collect();
        
        for activities in &mut activities {
            let start = activities[0].id;    
            for i in 0..activities.len() {
                activities[i].precedences = (start..(start+i)).collect();
            }
        }

        let jobs = activities.iter().map(|x| x.iter().map(|x| x.id).collect()).collect();
        let activities = activities.into_iter().flatten().collect();

        Ok(Problem {
            machines, activities, jobs, optimal,
        })
    }
    pub fn into_graph<I: Graph>(&self) -> I {
        let problem = self;
	    // Create nodes
		let mut nodes: Vec<I::Node> = Vec::new();
		let mut counter = 0;
		nodes.push(I::Node::create(0, 0, None, None));

		for (job_id, activities) in problem.jobs.iter().enumerate() {
			nodes.extend(activities.iter()
				.map(|x| {
					counter += 1;
					let id = counter;
					let weight = problem.activities[*x].process_time;
					let machine_id = problem.activities[*x].machine_id;
					I::Node::create(id, weight, Some(machine_id), Some(job_id))
				}));
		}
		nodes.push(I::Node::create(nodes.len(), 0, None, None));		

		let mut edges: Vec<Vec<Relation>> = nodes.iter().map(|_| Vec::new()).collect();		
		
		// Add edges within job
		for activities in &problem.jobs {
			for activity in activities.windows(2) {
				// node_1 -> node_2
				// Reminder that the indices within [activitys] point to the nodes within the problem.
				// Not to the graph nodes. graph nodes have to extra nodes: sink and source
				let node_1 = activity[0] + 1; // Skip the source node
				let node_2 = activity[1] + 1; // Skip the source node
				edges[node_1].push(Relation::Successor(node_2));
				edges[node_2].push(Relation::Predecessor(node_1));
			}

			// Add source edges
			let n = nodes.len() - 1;
			let first = activities[0] + 1;
			let last = activities.last().unwrap() + 1;
			edges[0].push(Relation::Successor(first));
			edges[first].push(Relation::Predecessor(0));

			// Add sink edges			
			edges[n].push(Relation::Predecessor(last));
			edges[last].push(Relation::Successor(n));
		}

		// Add source edges
		
		// Add disjunctions between activities:
		// - on the same machine
		// - not on the same job
		type Activity = (usize, usize); // (Job, node_index)		
		let mut mapping: Vec<Vec<Activity>> = vec!(vec!(); problem.machines as usize);
		for (job, activities) in problem.jobs.iter().enumerate() {
			for activity in activities {
				let machine = problem.activities[*activity].machine_id as usize;				
				mapping[machine - 1].push((job, activity + 1));
			}
		}

		for activities in mapping {
			for i in 0..(activities.len() - 1) {
				for j in (i+1)..activities.len() {
					let ac_1 = activities[i];
					let ac_2 = activities[j];

					if ac_1.0 != ac_2.0 {
						edges[ac_1.1].push(Relation::Disjunctive(ac_2.1));
						edges[ac_2.1].push(Relation::Disjunctive(ac_1.1));
					}
				}
			}
		}
		
		I::create(nodes, edges)
}
}

#[derive(Clone, Debug)]
pub struct ProblemNode {
    id: usize,
    weight: u32,
    job_id: Option<usize>,
    machine_id: Option<u32>
}

impl disjunctgraph::NodeId for ProblemNode {
    fn id(&self) -> usize { self.id }
}

impl disjunctgraph::GraphNode for ProblemNode {
    fn create(id: usize, weight: u32, machine_id: Option<u32>, job_id: Option<usize>) -> Self {
        ProblemNode {
            id, weight, job_id, machine_id
        }
    }
    fn weight(&self) -> u32 { self.weight }
    fn job_id(&self) -> Option<usize> { self.job_id }
    fn machine_id(&self) -> Option<u32> {self.machine_id }
}


