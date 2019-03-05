use std::rc::{ Weak, Rc };
use std::collections::VecDeque;

use crate::problem::{ Problem, Job, Activity };
use crate::schedule::OrderedJobs;

fn solve_greedy(problem: &Problem) -> OrderedJobs {
    let machines = problem.machines;
    let jobs = &problem.jobs;

    panic!()
}