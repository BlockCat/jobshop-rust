use std::collections::{ HashSet, HashMap };
use disjunctgraph::{ NodeId, NodeWeight, Graph, self};

use crate::problem::Problem;

// This is a disjunctive graph.
// No edges should be added after creation.
// Disjunctions can be fixed, this will return a new instance of the graph.
// Would be nice if two nodes could reference the same edge, then a change in the edge would result in a change in the nodes.
// Switch to:
// https://ac.els-cdn.com/S0377221799004865/1-s2.0-S0377221799004865-main.pdf?_tid=b48d4410-024d-4d81-9762-34e0b7b924a4&acdnat=1552055948_805fca074e1e30d2c269cbf795004fd4
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

impl NodeId for Node {
	fn id(&self) -> usize { self.id }
}

impl NodeWeight for Node {
	fn weight(&self) -> u32 { self.processing_time }
}

impl Graph<Node> for DisjunctiveGraph {
	fn nodes(&self) -> &[Node] {
		&self.nodes
	}

	fn source(&self) -> &Node {
		unimplemented!()
	}

	fn sink(&self) -> &Node {
		unimplemented!()
	}

    fn successors(&self, id: &impl NodeId) -> Vec<&Node> {
		unimplemented!()
	}

    fn predecessors(&self, id: &impl NodeId) -> Vec<&Node> {
		unimplemented!()
	}
    fn disjunctions(&self, id: &impl NodeId) -> Vec<&Node> {
		unimplemented!()
	}
    fn fix_disjunction(&self, node_1: &impl NodeId, node_2: &impl NodeId) -> disjunctgraph::Result<Self> {
		unimplemented!()
	}
    fn flip_edge(&self, node_1: &impl NodeId, node_2: &impl NodeId) -> disjunctgraph::Result<Self> {
		unimplemented!()
	}
    fn into_directed(&self) -> disjunctgraph::Result<Self> {
		unimplemented!()
	}
}

impl From<&Problem> for DisjunctiveGraph {
	fn from(problem: &Problem) -> Self {

		// Create a node for every activity, still grouped by job and in order
		let mut counter = 0;
		let mut jobs: Vec<Vec<Node>> = problem.jobs.iter().enumerate()
			.map(|(job_id, activities)| {
				// For every activity create a node
				activities.iter()                    
					.map(|id| &problem.activities[*id])
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
			for (i, (job_1, ac1)) in (0..(activities.len() - 1)).map(|x| (x, activities[x])) {
				let others = ((i+1)..activities.len())
					.map(|x| activities[x])
					.filter(|(_,   ac2)| &ac1 != ac2 ) // Different activity
					.filter(|(job_2, _)| &job_1 != job_2); // Different job

				for (_, ac2) in others {
					nodes[ac1].disjunctions.insert(ac2);
					nodes[ac2].disjunctions.insert(ac1);	
				}
			}
		}

		DisjunctiveGraph {
			nodes
		}
	}
}
/*
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
		self.nodes().iter().map(|x| x.id()).collect()
	}

	fn edges(&'a self) -> dot::Edges<'a, DotEdge> {
		use dot::{ ArrowShape, Fill, Side };
		let directed = ArrowShape::Normal(Fill::Filled, Side::Both);
		let undirected = ArrowShape::NoArrow;

		let successors = self.nodes()
			.iter()
			.flat_map(|x| self.successors(x).iter().map(|s| (x, s)))
			.map(|(a, b)| (directed, a.id(), b.id()));
		
		let disjunctions = self.nodes()
			.iter()
			.flat_map(|x| self.disjunctions(x).iter().map(|s| (x, s)))
			.filter(|(a, b)| b.id() > a.id())
			.map(|(a, b)| (undirected, a.id(), b.id()));

		successors.chain(disjunctions).collect()		
	}

	fn source(&self, e: &DotEdge) -> DotNode { e.1 }
    fn target(&self, e: &DotEdge) -> DotNode { e.2 }
}*/