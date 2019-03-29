use crate::problem::{ Activity, Problem, ProblemNode };

use disjunctgraph::{Graph, NodeId, GraphNode };

#[derive(Debug, PartialEq, Eq)]
pub struct ScheduledActivity {    
    pub starting_time: u32,
    pub activity: Activity
}

#[derive(Debug, PartialEq, Eq)]
pub struct Schedule {
    pub activities: Vec<ScheduledActivity>,  
    pub jobs: Vec<Vec<usize>>
}

pub struct OrderedActivities {
    pub problem: Problem,    
    pub jobs: Vec<Vec<usize>>
}

impl Schedule {
    pub fn length() -> u32 {
        unimplemented!();
    }


    pub fn from_graph<I: Graph>(problem: Problem, graph: I) -> Schedule {
        use std::collections::VecDeque;

        let topology = graph.topology();
        let mut nodes = (0..graph.nodes().len()).collect::<Vec<usize>>();
        nodes.sort_by_key(|x| std::cmp::Reverse(topology[*x]));

        // Starting with the node with the highest topology, the source...
        let mut starting_times = vec!(0u32; nodes.len());
        let mut backtracker = vec!(0usize; nodes.len());

        for node in nodes {
            let predecessors = graph.predecessors(&node);

            let nodes = graph.nodes();
            let max_predecessor = predecessors.iter()                
                .map(|x| (x.id(), starting_times[x.id()] + nodes[x.id()].weight()))
                .max_by_key(|x| x.1);

            if let Some(max_predecessor) = max_predecessor {
                backtracker[node] = max_predecessor.0;
                starting_times[node] = max_predecessor.1;
            }
        }
        
        
        let jobs = problem.jobs.clone();
        let activities = problem.activities.into_iter().enumerate()
            .map(|(i, activity)| {
                ScheduledActivity {
                    activity,
                    starting_time: starting_times[i + 1]
                }
            });

        Schedule {
            jobs,
            activities: activities.collect()
        }
    }

}

impl From<OrderedActivities> for Schedule {
    fn from(ordered_activities: OrderedActivities) -> Schedule {
        
        // Recursive function that calculates
        // s_i = max_{j\in P} (s_j + p_j)
        #[derive(Debug)]
        struct ActivityOption {
            processing_time: u32,
            precedences: Vec<usize>
        }
        fn si (index: usize, activity_times: &mut Vec<Option<u32>>, precedences: &[ActivityOption]) -> u32 {            
            match activity_times[index] {
                Some(e) => e,
                None => {
                    //let precedents = vec!();                    
                    let max = precedences[index].precedences.iter()
                        .map(|x| si(*x, activity_times, precedences) + precedences[*x].processing_time)
                        .max()
                        .unwrap_or(0u32);
                    
                    activity_times[index] = Some(max);
                    max
                }
            }
        };

        let activities = &ordered_activities.problem.activities;
        let mut activity_times: Vec<Option<u32>> = activities.iter().map(|_| None).collect::<Vec<_>>();
        
        let precedences: Vec<ActivityOption> = { 
            // Ordering of activities on machines.
            let jobs = &ordered_activities.jobs; 
            // List of precedence constraints for every activity, includes precedents by machine.
            let mut activity_precedences = activities.iter()
                .map(|x| Vec::with_capacity(x.precedences.len() + 1))
                .collect::<Vec<_>>();
            
            // Fill in precedences
            for indeces in jobs.iter() {
                let mut prev = None;
                for i in indeces {
                    let mut precedences = activities[*i].precedences.clone();
                    if let Some(prev) = prev {
                        precedences.push(prev);
                    }
                    prev = Some(*i);
                    activity_precedences[*i].extend(precedences);
                }
            }
            activity_precedences.into_iter().enumerate()
                .map(|(i, precedences)| {
                    ActivityOption {
                        processing_time: activities[i].process_time,
                        precedences
                    }
                }).collect()
        };

        let jobs = ordered_activities.jobs.clone();
        let activities = (0..activities.len()).map(|x| {
            si(x, &mut activity_times, &precedences)
        }).zip(ordered_activities.problem.activities).map(|(starting_time, activity)| {
            ScheduledActivity {
                starting_time, activity
            }
        });

        Schedule {
            jobs: jobs,
            activities: activities.collect()
        }
    }
}