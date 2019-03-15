use crate::problem::{ Problem, ProblemNode };
use disjunctgraph::{ Graph, Relation };

pub struct ProblemGraph<I>(pub I);

impl<I: Graph<ProblemNode>> From<&Problem> for ProblemGraph<I> {

	fn from(problem: &Problem) -> ProblemGraph<I> {

		// Create nodes
		let mut nodes: Vec<ProblemNode> = Vec::new();
		nodes.push(ProblemNode {
			id: 0,
			weight: 0,			
		});
		nodes.extend(problem.jobs.iter().flatten().enumerate()
			.map(|(k, x)| {				
				ProblemNode {
					id: k + 1,
					weight: problem.activities[*x].process_time,
				}
			})
		);
		nodes.push(ProblemNode {
			id: nodes.len(),
			weight: 0
		});

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
		
		ProblemGraph(I::create(nodes, edges))
	}
}