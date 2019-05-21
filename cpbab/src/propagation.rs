use disjunctgraph::{ Graph, ConstrainedNode, GraphNode, NodeId };

pub fn propagate<I: Graph>(graph: &mut I, node_1: &impl NodeId, node_2: &impl NodeId) -> Result<(), ()>
    where I::Node: ConstrainedNode {
        use std::collections::VecDeque;
        
        let mut change_occured = true;

        
        enum Propagation {
            LST { id: usize, max_lst: u32},
            EST { id: usize, min_est: u32}
        }        
        let mut stack: VecDeque<Propagation> = VecDeque::with_capacity(graph.nodes().len());
        
        while change_occured {
            stack.clear();
            // Add the successors
            let n2_est = graph[node_2.id()].est() + graph[node_2.id()].weight();
            for successor in graph.successors(node_2) {
                stack.push_back(Propagation::EST {
                    id: successor.id(),
                    min_est: n2_est
                });
            }

            let n1_lst = graph[node_1.id()].lst();
            for predecessor in graph.predecessors(node_1) {
                stack.push_back(Propagation::LST {
                    id: predecessor.id(),
                    max_lst: n1_lst
                });
            }

            while let Some(prop) = stack.pop_front() {
                match prop {
                    Propagation::LST { id, max_lst } => {
                        let node = &graph[id];
                        if max_lst < node.lst() {
                            if node.feasible_lst(max_lst) {
                                stack.extend(graph.successors(&id).map(|s| {
                                    Propagation::LST {
                                        id: s.id(),
                                        max_lst: max_lst + node.weight()
                                    }
                                }));
                                graph[id].set_lct(max_lst);
                            } else {
                                return Err(());
                            }
                        }
                    },
                    Propagation::EST { id, min_est } => {
                        let node = &graph[id];
                        if min_est > node.est() {
                            if node.feasible_est(min_est) {
                                stack.extend(graph.predecessors(&id).map(|s| {
                                    Propagation::EST {
                                        id: s.id(),
                                        min_est: min_est - s.weight()
                                    }    
                                }));
                                graph[id].set_est(min_est);
                            } else {
                                return Err(());
                            }
                        }
                    }
                }
            }
            change_occured = graph.search_orders();
        }

        Ok(())
    }