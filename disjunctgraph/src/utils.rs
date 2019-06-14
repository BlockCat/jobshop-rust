use crate::{ ConstrainedNode, Graph, NodeId, GraphNode };
use itertools::Itertools;

fn find_orders<T: Graph>(resources: &[u32], graph: T) -> Result<Vec<(usize, usize)>, ()> where T::Node: ConstrainedNode {

    let orders = Vec::new();

    for resource in resources {
        let resource = graph.nodes().iter().filter(|n| n.machine_id() == Some(*resource)).collect_vec();
        let heads = resource.iter().sorted_by_key(|n| n.head());
        let tails = resource.iter().sorted_by_key(|n| n.tail());
        
        

    }

    Ok(orders)
}