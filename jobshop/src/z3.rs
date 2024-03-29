use crate::problem::{ ProblemSolver, Problem };
use disjunctgraph::Graph;

// In the case of a search, it might be nice to only store partial orientations.
// As described in https://pure.tue.nl/ws/files/2119953/385216.pdf
pub struct Z3Solve;

impl Z3Solve {
    pub fn new() -> Self {
        Z3Solve {}
    }
}
impl ProblemSolver for Z3Solve {
    type Solution = u32;

    fn solve(&self, problem: &Problem) -> Self::Solution {
        let graph = problem.into_graph();
        let solution = z3solver::solve(graph);

        for st in solution.iter().enumerate() {
            println!("node-{} starts at {}", st.0, st.1);
        }

        // let ordered = OrderedActivities {

        // };

        *solution.last().unwrap()
    }
}


#[cfg(test)]
mod tests {    
    use crate::z3::Z3Solve;
    use crate::problem::{ Problem, ProblemSolver };

    #[test]
    fn z3solver_1() {
        let problem = debug_problem();
        let z3solver = Z3Solve::new();
        assert_eq!(13, z3solver.solve(&problem));
    }

    #[test]
    fn z3solver_2() {
        let problem = small_problem();
        let z3solver = Z3Solve::new();
        assert_eq!(13, z3solver.solve(&problem));
    }

    #[test]
    fn z3solver_3() {
        let problem = dmu03_rcmax_20_15_5();
        let z3solver = Z3Solve::new();
        assert_eq!(2731, z3solver.solve(&problem));
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

    fn dmu03_rcmax_20_15_5() -> Problem {
        Problem::from_reader(r"20
15
2731
84 119 128 144 177 151 138 16 195 93 107 22 137 96 21
95 91 153 109 182 47 98 54 159 123 5 5 141 79 160
91 62 173 67 136 140 115 183 186 6 190 173 139 28 183
119 188 43 18 23 58 136 54 194 35 40 32 184 112 186
199 13 63 58 55 82 22 183 43 157 25 60 150 12 115
113 109 185 59 3 24 71 98 32 102 19 20 112 14 39
194 133 117 13 111 126 101 38 184 135 99 92 146 44 158
103 93 21 148 66 29 11 4 28 93 192 67 96 16 64
124 185 153 143 30 27 69 130 53 189 86 78 155 87 114
168 5 17 186 133 35 101 172 56 126 75 93 67 109 127
90 199 185 94 40 92 146 90 131 57 135 190 192 56 103
45 45 157 13 126 44 152 148 122 158 148 103 69 93 192
107 137 14 113 138 182 179 107 118 172 157 178 127 34 82
9 163 104 20 21 48 131 9 125 101 106 195 161 74 115
187 55 76 56 59 11 74 2 194 13 104 147 166 34 118
45 170 135 72 56 146 190 57 148 39 163 14 168 101 99
104 75 183 152 166 10 122 32 94 161 150 1 98 113 26
42 62 86 118 128 153 134 156 80 105 16 186 84 42 129
160 119 93 22 168 138 162 65 56 171 20 8 137 193 92
185 19 11 87 171 14 65 61 34 93 154 67 14 136 27
12 13 3 4 9 1 10 7 15 6 14 2 11 5 8
15 1 9 7 3 11 8 12 5 10 14 13 6 2 4
14 4 5 12 1 11 13 3 15 6 2 7 10 9 8
14 6 4 9 13 15 3 1 7 2 5 8 10 12 11
2 12 11 9 5 10 6 13 4 1 15 14 7 8 3
7 4 6 9 2 10 1 15 14 11 13 12 5 8 3
14 12 13 15 8 3 9 7 1 10 2 11 6 4 5
7 5 13 11 6 4 2 10 14 3 12 1 9 8 15
8 12 7 6 1 9 3 10 13 5 15 11 2 14 4
15 8 5 10 7 1 14 9 4 11 3 2 13 12 6
5 4 13 9 7 1 3 8 10 6 14 12 2 11 15
5 1 8 15 2 12 6 11 3 7 9 14 4 13 10
8 7 3 10 2 9 5 12 14 6 15 13 4 1 11
15 3 11 14 1 4 2 12 7 10 5 9 8 6 13
8 5 6 3 2 13 14 9 4 10 11 7 12 1 15
8 6 4 7 12 1 9 3 14 15 13 2 5 10 11
9 8 1 15 3 11 12 14 7 10 13 4 2 5 6
5 15 14 1 13 3 9 7 12 6 8 4 2 11 10
8 2 15 4 9 13 14 6 7 10 12 3 5 1 11
13 9 4 8 12 5 14 2 1 10 11 7 3 6 15
".as_bytes()).unwrap()
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