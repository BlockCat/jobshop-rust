use crate::{ NodeId, NodeWeight, Graph, self as disjunctgraph };

// This will be an N x (N + 3) matrix. (N includes source and sink)
// As an adjustment of this:
// https://ac.els-cdn.com/S0377221799004865/1-s2.0-S0377221799004865-main.pdf?_tid=bc7e7478-9c03-4eee-b93e-4d793c0aca48&acdnat=1552300381_a73c9971ebc05ade30ef09824ed52d9f
// A cell ranges from: -2n to 2n
// In the first matrix from columns 0 to n
// n  <  x <= 2n => Successor and (x-n) is the next successor
// 0  <= x <= n  => Predecessor and x is the next predecessor
// -n <= x < 0   => Disjunctive relationship and -x is the next disjunction
// -2n <= x < -n => No relationship and (x + n) is the next unkown relationship

// Node 0 is the source
// Node (n-1) is the sink
// Column (n+1) references to the first successor
// Column (n+2) references to the first predecessor
// Column (n+3) references to the first disjunction

#[derive(Clone)]
pub struct MatrixGraph<T: NodeId + Clone> {
    nodes: Vec<T>,    
    matrix: Vec<Vec<i32>>,
    x_dim: usize,
}

impl<T: NodeId + NodeWeight + Clone> Graph<T> for MatrixGraph<T> {
    fn nodes(&self) -> &[T] {
		&self.nodes
	}

	fn source(&self) -> &T {
		&self.nodes().first().unwrap()
	}

	fn sink(&self) -> &T {
		&self.nodes().last().unwrap()
	}

    fn successors<'a>(&'a self, id: &impl NodeId) -> Vec<&T> {
        MatrixIterator::<'a> {
            matrix: &self.matrix,
            id: id.id(),
            nodes: self.nodes.len() as isize,
            column: self.x_dim - 3
        }.map(|x| &self.nodes[x]).collect()
	}

    fn predecessors<'a>(&'a self, id: &impl NodeId) -> Vec<&T> {
		MatrixIterator::<'a> {
            matrix: &self.matrix,
            id: id.id(),
            nodes: 0,
            column: self.x_dim - 2
        }.map(|x| &self.nodes[x]).collect()
	}
    fn disjunctions<'a>(&'a self, id: &impl NodeId) -> Vec<&T> {
		MatrixIterator::<'a> {
            matrix: &self.matrix,
            id: id.id(),
            nodes: -(self.nodes.len() as isize),
            column: self.x_dim - 1
        }.map(|x| &self.nodes[x]).collect()
	}

    fn fix_disjunction(&self, node_1: &impl NodeId, node_2: &impl NodeId) -> disjunctgraph::Result<Self> {
        let mut cloned = self.clone();
        let node_1 = node_1.id();
        let node_2 = node_2.id();
        
        // Remove from disjunctions
        // Remove from (node_1, node_2)
        // Remove from (node_2, node_1)
        // Add node_2 as successor of node_1
        // Add node_1 as predecessor of node_2
		unimplemented!()
	}

    fn flip_edge(&self, node_1: &impl NodeId, node_2: &impl NodeId) -> disjunctgraph::Result<Self> {
        // Remove node_1 from predecessors of node_2
        // Remove node_2 from predecessors of node_1
        // Add node_1 as successor of node_2
        // Add node_2 as predecessor of node_1
		unimplemented!()
	}

    fn into_directed(&self) -> disjunctgraph::Result<Self> {
        // For every disjunction, flip edge
		unimplemented!()
	}
}

struct MatrixIterator<'a> {
    matrix: &'a Vec<Vec<i32>>,
    id: usize,
    nodes: isize,
    column: usize
}

impl<'a> Iterator for MatrixIterator<'a> {
    type Item = usize;

    fn next(&mut self) -> Option<usize> {
        let old_column = self.column as isize;
        let next_column = self.matrix[self.id][self.column] as isize - self.nodes;
        
        if old_column == next_column {
            None
        } else {
            self.column = next_column as usize;
            Some(self.column)
        }
    }
}