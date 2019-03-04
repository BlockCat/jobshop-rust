use std::io::{ BufRead, BufReader };
use std::fs::File;
use std::path::Path;
use std::rc::{ Rc, Weak };
use std::cell::RefCell;

use crate::schedule::Schedule;

pub trait ProblemSolver {
    fn solve() -> Schedule;
}

#[derive(Debug)]
pub struct Problem {
    machines: u32,
    jobs: Vec<Job>,
    optimal: u32,
}

#[derive(Debug)]
pub struct Job(Vec<Rc<Activity>>);

#[derive(Debug)]
pub struct Activity {
    process_time: u32,
    machine_id: u32,
    precedences: RefCell<Vec<Weak<Activity>>>,    
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

        // Take jobs processing times
        let processing_times = reader.by_ref()
            .take(jobs as usize)
            .map(|x| x.split(" ").map(|x| x.parse::<u32>().unwrap()).collect::<Vec<_>>())
            .collect::<Vec<_>>().into_iter();

        let machine_placements = reader.by_ref()
            .take(jobs as usize)
            .map(|x| x.split(" ").map(|x| x.parse::<u32>().unwrap()).collect::<Vec<_>>())
            .collect::<Vec<_>>().into_iter();
            
        let jobs = processing_times.zip(machine_placements)
            .map(|(p, m)| {
                p.into_iter().zip(m.into_iter()).collect::<Vec<_>>()
            })
            .map(|activities| {
                let mut activities = activities.into_iter().map(|x| Rc::new(Activity {
                    process_time: x.0,
                    machine_id: x.1,
                    precedences: RefCell::default(),
                })).collect::<Vec<_>>();


                for k in 0..activities.len() {
                    let precedences = activities.iter()
                        .take(k)
                        .map(|x| Rc::downgrade(x))
                        .collect::<Vec<_>>();                    
                    *activities[k].precedences.borrow_mut() = precedences;    
                }                    
                
                Job(activities)
            }).collect();
       

        Ok(Problem {
            machines, jobs, optimal
        })
    }
}
