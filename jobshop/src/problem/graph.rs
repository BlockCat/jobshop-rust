use crate::problem::{ Problem, ProblemNode };
use disjunctgraph::{ Graph, Relation };

pub struct ProblemGraph<I>(pub I);

impl<I: Graph<ProblemNode>> From<&Problem> for ProblemGraph<I> {

	fn from(problem: &Problem) -> ProblemGraph<I> {

		// Create nodes
		
		let source = ProblemNode { id: counter, weight: 0 };		
		let nodes: Vec<ProblemNode> = problem.jobs.iter().flatten()
			.map(|x| {
				counter += 1;
				ProblemNode {
					id: counter,
					weight: problem.activities[*x].process_time,
				}
			}).collect();
		let sink = ProblemNode { id: counter + 1, weight: 0 };
		
		let mut edges: Vec<Vec<Relation>> = nodes.iter().map(|_| Vec::new()).collect();		
		
		
		ProblemGraph(I::create(nodes, edges))
	}
}

/*
impl From<&Problem> for LinkedGraph {
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
}*/


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