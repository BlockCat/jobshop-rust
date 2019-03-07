
use std::collections::HashSet;


use crate::problem::{ Activity, Problem };

pub struct Schedule {
    problem: Problem,
    jobs: Vec<Vec<(u32, usize)>>
}

pub struct OrderedActivities {
    problem: Problem,    
    jobs: Vec<Vec<usize>>
}


impl From<OrderedActivities> for Schedule {
    fn from(mut ordered_activities: OrderedActivities) -> Schedule {

        let mut processed: HashSet<usize> = HashSet::with_capacity(ordered_activities.problem.activities.len());
        let machine_times = (0..ordered_activities.problem.machines).map(|_| 0u32).collect::<Vec<_>>();

        // s_i = max_{j\in P} (s_j + p_j)
        
        panic!()
    }
}