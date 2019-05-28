#![feature(test)]
extern crate test;

pub mod problem;
pub mod local_search;
pub mod cpbab;
//pub mod branch_and_bound;
//pub mod constraints;
pub mod schedule;

#[cfg(test)]
mod tests {    
    use crate::cpbab::CPBAB;
    use crate::problem::{ Problem, ProblemSolver };
    use test::Bencher;

    #[test]
    fn test_cpbab_1() {
        use disjunctgraph::Graph;
        let problem = debug_problem();
        let l = CPBAB::new().solve(&problem);

        let schedule = crate::schedule::Schedule::from_graph(problem, l.clone());        
        println!("Completed: {}", !l.has_disjunctions());
        schedule.pretty_print();
        assert_eq!(13, l.critical_length().unwrap());
    }


    #[test]
    fn test_cpbab_2() {
        use disjunctgraph::Graph;
        let problem = small_problem();
        let l = CPBAB::new().solve(&problem);

        let schedule = crate::schedule::Schedule::from_graph(problem, l.clone());        
        println!("Completed: {}", !l.has_disjunctions());
        schedule.pretty_print();
        assert_eq!(13, l.critical_length().unwrap());
    }
    
    #[test]
    fn test_cpbab_3() {
        use disjunctgraph::Graph;
        let problem = big_problem();
        let l = CPBAB::new().solve(&problem);

        let schedule = crate::schedule::Schedule::from_graph(problem, l.clone());        
        println!("Completed: {}", !l.has_disjunctions());
        schedule.pretty_print();
        assert_eq!(593, l.critical_length().unwrap());
    }

    /*#[bench]
    fn bench_local_search_small(b: &mut Bencher) {
        use crate::local_search::LocalSearch;
        let problem = small_problem();
        let solver = LocalSearch::new(20);

        b.iter(|| solver.solve(&problem));
    }

    #[bench]
    fn bench_local_search_large(b: &mut Bencher) {
        use crate::local_search::LocalSearch;
        let problem = big_problem();
        let solver = LocalSearch::new(10000);

        b.iter(|| solver.solve(&problem));
    }*/

    fn debug_problem() -> Problem {
        Problem::from_reader(r"2
2
13
2 7 2
4 3
1 2 1
2 1".as_bytes()).unwrap()
    }
    fn small_problem() -> Problem {
        Problem::from_reader(r"3
3
13
3 2 3
3 4
6 3 2
1 2 3
3 2
2 1 3".as_bytes()).unwrap()
    }

    fn big_problem() -> Problem {
        Problem::from_reader(r"10
5
593
87 72 66 60 95
48 54 39 35 5
97 46 21 20 55
59 34 37 19 46
28 24 73 25 23
45 78 83 28 5
53 37 12 71 29
38 55 87 33 12
48 40 49 83 7
90 23 65 17 27
2 1 5 3 4
5 4 1 3 2
2 4 3 1 5
1 4 5 2 3
5 3 4 2 1
4 1 5 2 3
1 4 2 5 3
5 3 4 2 1
3 4 2 1 5
3 4 1 5 2
".as_bytes()).unwrap()
    }
}