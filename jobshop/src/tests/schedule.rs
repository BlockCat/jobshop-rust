use crate::problem::*;
use crate::schedule::*;

#[test]
fn from_ordered_jobs() {
    let ordering = ordering_1();
    let true_schedule = schedule_1();
    let new_schedule: Schedule = ordering.into();

    assert_eq!(true_schedule, new_schedule);    
}

//https://acrogenesis.com/or-tools/documentation/user_manual/_images/schedule1.png
fn simple_problem() -> Problem {
    let ac0 = Activity {id: 0, process_time: 3, machine_id: 1, precedences: vec!()};
    let ac1 = Activity {id: 1, process_time: 2, machine_id: 2, precedences: vec!(0)};
    let ac2 = Activity {id: 2, process_time: 2, machine_id: 3, precedences: vec!(0, 1)};

    let ac3 = Activity {id: 3, process_time: 2, machine_id: 1, precedences: vec!()};
    let ac4 = Activity {id: 4, process_time: 1, machine_id: 3, precedences: vec!(3)};
    let ac5 = Activity {id: 5, process_time: 4, machine_id: 2, precedences: vec!(3, 4)};

    let ac6 = Activity {id: 6, process_time: 4, machine_id: 2, precedences: vec!()};
    let ac7 = Activity {id: 7, process_time: 3, machine_id: 3, precedences: vec!(6)};
    
    Problem {
        machines: 3,
        activities: vec!(ac0, ac1, ac2, ac3, ac4, ac5, ac6, ac7),
        jobs: vec!(vec!(0, 1, 2), vec!(3, 4, 5), vec!(6, 7)),
        optimal: 12,
    }
}

fn ordering_1() -> OrderedActivities {
    OrderedActivities {
        problem: simple_problem(),
        jobs: vec!(
            vec!(3, 0),
            vec!(6, 5, 1),
            vec!(4, 7, 2),
        ),
    }
}

fn schedule_1() -> Schedule {
    let activities = simple_problem().activities;
    let jobs = ordering_1().jobs;
    let mut activities = activities.into_iter().map(|x| ScheduledActivity {
        starting_time: 0,
        activity: x
    }).collect::<Vec<_>>();

    activities[0].starting_time = 2;
    activities[1].starting_time = 8;
    activities[2].starting_time = 10;
    activities[3].starting_time = 0;
    activities[4].starting_time = 2;
    activities[5].starting_time = 4;
    activities[6].starting_time = 0;
    activities[7].starting_time = 4;

    Schedule {
        activities: activities,  
        jobs: jobs
    }
}