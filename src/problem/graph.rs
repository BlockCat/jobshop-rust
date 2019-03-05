use std::collections::{ HashSet, HashMap };

use crate::problem::Problem;

// This is a disjunctive graph.
// No edges should be added after creation.
// Disjunctions can be fixed, this will return a new instance of the graph.
// Would be nice if two nodes could reference the same edge, then a change in the edge would result in a change in the nodes.

#[derive(Debug, Clone)]
pub struct DisjunctiveGraph {
	nodes: Vec<Node>
}

#[derive(Debug, Clone)]
pub struct Node {
	id: usize,
	job_id: u32,
	machine_id: u32,
	processing_time: u32,
	predecessors: HashSet<usize>,
	successors: HashSet<usize>,
	disjunctions: HashSet<usize>    
}

impl From<&Problem> for DisjunctiveGraph {
	fn from(problem: &Problem) -> Self {

		// Create a node for every activity, still grouped by job and in order
		let mut counter = 0;
		let mut jobs = problem.jobs.iter().enumerate()
			.map(|(job_id, activities)| {
				// For every activity create a node
				activities.0.iter()                    
					.map(|activity| {
						let id = counter;
						counter += 1;
						Node {
							id, 
							job_id: job_id as u32,
							machine_id: activity.machine_id,
							processing_time: activity.process_time,
							predecessors: HashSet::new(),
							successors: HashSet::new(),
							disjunctions: HashSet::new(),
						}
					}).collect::<Vec<_>>()
			}).collect::<Vec<_>>();

		// Create a mapping for which Activity on which machine
		let mut mapping: HashMap<u32, Vec<(usize, usize)>> = HashMap::with_capacity(problem.machines as usize); // machine_id -> [(job_index, activity_index)]
		for (job_id, activities) in jobs.iter().enumerate() {
			for activity in activities.iter() {
				mapping.entry(activity.machine_id)
					.and_modify(|x| x.push((job_id, activity.id)))
					.or_insert(vec!((job_id, activity.id)));
			}
		}

		// Create arcs between activities in the same job.
		for activities in &mut jobs {
			let start_node = activities[0].id;
			let last_node = activities.last().unwrap().id;

			for node in start_node..last_node {                
				activities[node - start_node].successors.insert(node + 1);                
				activities[node + 1 - start_node].predecessors.insert(node);
			}
		}

		let mut nodes: Vec<Node> = jobs.into_iter().flatten().collect();

		// Search for activities on other jobs with the same machine.
		for activities in mapping.values() {
			for i in 0..(activities.len() - 1) {
				let (job_1, ac1) = activities[i];
				for j in (i+1)..activities.len() {
					let (job_2, ac2) = activities[j];
					if job_1 != job_2 && ac1 != ac2 { // Different job and different activity
						nodes[ac1].disjunctions.insert(ac2);
						nodes[ac2].disjunctions.insert(ac1);						
					}
				}
			}
		}

		DisjunctiveGraph {
			nodes
		}
	}
}


type DotNode = usize;
type DotEdge = (dot::ArrowShape, DotNode, DotNode);
impl<'a> dot::Labeller<'a, DotNode, DotEdge> for DisjunctiveGraph {
	fn graph_id(&'a self) -> dot::Id<'a> {
		dot::Id::new("G").unwrap()
	}
	
	fn node_id(&'a self, n: &DotNode) -> dot::Id<'a> {
        dot::Id::new(format!("N{}", n)).unwrap()
    }

    fn node_label<'b>(&'b self, n: &DotNode) -> dot::LabelText<'b> {
        dot::LabelText::LabelStr(format!("{}", n).into())
    }

    fn edge_label<'b>(&'b self, _: &DotEdge) -> dot::LabelText<'b> {
        dot::LabelText::LabelStr("".into())
    }

	fn edge_end_arrow(&'a self, e: &DotEdge) -> dot::Arrow {
		dot::Arrow::from_arrow(e.0)
	}
}

impl<'a> dot::GraphWalk<'a, DotNode, DotEdge> for DisjunctiveGraph {

	fn nodes(&self) -> dot::Nodes<'a, DotNode> {
		self.nodes.iter().map(|x| x.id).collect()
    }

    fn edges(&'a self) -> dot::Edges<'a, DotEdge> {
		use dot::{ ArrowShape, Fill, Side };
		let directed = ArrowShape::Normal(Fill::Filled, Side::Both);		
		let directed = self.nodes.iter()
			.flat_map(|x| {
				x.successors.iter()
					.map(move |s| (directed, x.id, *s))
			});
		
		let undirected = ArrowShape::NoArrow;
		let undirected = self.nodes.iter()
			.flat_map(|x| {
				x.disjunctions.iter()
					.filter(move |s| s > &&x.id )
					.map(move |s| (undirected, x.id, *s))
			
			});

		let mut edges = Vec::with_capacity(self.nodes.len() * 2);

		edges.extend(directed);
		edges.extend(undirected);

		std::borrow::Cow::Owned(edges)
    }

    fn source(&self, e: &DotEdge) -> DotNode { e.1 }

    fn target(&self, e: &DotEdge) -> DotNode { e.2 }
}

impl DisjunctiveGraph {

	// Any node that does not have precedences
	fn starting_nodes<'a> (&'a self) -> Vec<&'a Node> {
		self.nodes.iter()
			.filter(|x| x.predecessors.is_empty())
			.collect()
	}

	fn ending_nodes(&self) -> Vec<&Node> {
		self.nodes.iter()
			.filter(|x| x.successors.is_empty())
			.collect()
	}

	fn fix_disjunction<'a>(&'a self, a: &'a Node, b: &'a Node) -> Result<DisjunctiveGraph, String> {

		if !a.disjunctions.contains(&b.id) || !b.disjunctions.contains(&a.id) {
			Err("".to_owned())
		} else {
			let mut cloned = self.clone();
			cloned.nodes[a.id].disjunctions.remove(&b.id);
			cloned.nodes[b.id].disjunctions.remove(&a.id);

			Ok(cloned)
		}
	}
}