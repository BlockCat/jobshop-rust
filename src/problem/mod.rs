use std::io::{ BufRead, BufReader };
use std::fs::File;
use std::path::Path;

use crate::schedule::Schedule;

mod greedy;
mod graph;

pub use graph::DisjunctiveGraph;

pub trait ProblemSolver {
    fn solve() -> Schedule;
}

#[derive(Debug, Clone)]
pub struct Problem {
    pub machines: u32,
    pub activities: Vec<Activity>,
    pub jobs: Vec<Vec<usize>>,
    pub optimal: u32,
}

#[derive(Debug, Clone)]
pub struct Activity {
    id: usize,
    pub process_time: u32,
    pub machine_id: u32,
    precedences: Vec<usize>,    
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
}