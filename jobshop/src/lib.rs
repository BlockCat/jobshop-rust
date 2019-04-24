#![feature(test)]
extern crate test;

pub mod problem;
pub mod local_search;
pub mod constraints;
pub mod schedule;



#[cfg(test)]
mod tests {
    use crate::local_search::LocalSearch;
    use crate::problem::{ Problem, ProblemSolver };
    use test::Bencher;

    #[bench]
    fn bench_local_search_small(b: &mut Bencher) {
        
        let problem = Problem::from_reader(r"3
3
13
3 2 3
3 4
6 3 2
1 2 3
3 2
2 1 3".as_bytes()).unwrap();
        let solver = LocalSearch::new(20);

        b.iter(|| solver.solve(&problem));
    }

    #[bench]
    fn bench_local_search_large(b: &mut Bencher) {
        
        let problem = Problem::from_reader(r"10
5
666
53 21 34 55 95
21 71 26 52 16
12 42 31 39 98
55 77 66 77 79
83 19 64 34 37
92 54 43 62 79
93 87 87 69 77
60 41 38 24 83
44 49 98 17 25
96 75 43 79 77
2 1 5 4 3
1 4 5 3 2
4 5 2 3 1
2 1 5 3 4
1 4 3 2 5
2 3 5 1 4
4 5 2 3 1
3 1 2 4 5
4 2 5 1 3
5 4 3 2 1
".as_bytes()).unwrap();
        let solver = LocalSearch::new(10000);

        b.iter(|| solver.solve(&problem));
    }

}