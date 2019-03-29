
extern crate gtk;
#[macro_use]
extern crate relm;
#[macro_use]
extern crate relm_derive;
extern crate relm_attributes;
extern crate cairo;

use relm_attributes::widget;

use disjunctgraph::{ Graph, LinkedGraph };
use jobshop::problem::*;
use jobshop::constraints::*;
use jobshop::local_search::LocalSearch;

use widget_graph::*;
use widget_constraints::*;
use widget_edge_selection::*;
use widget_edge_selection::EdgeMsg::Fix as EdgeFix;

use gtk::prelude::*;
use gtk::Orientation::{ Vertical, Horizontal };
use relm::Widget;

mod widget_graph;
mod widget_constraints;
mod widget_edge_selection;

const UPPER: u32 = 800;

#[derive(Msg)]
pub enum Msg {
    Decrement,
    Increment,
    Fix(usize, usize),
    Quit,
}
use Msg::Fix;

pub struct Model {
    counter: u32,
    graph: LinkedGraph<ProblemNode>,
    problem: Problem,
}

#[widget]
impl Widget for Win {

    fn model() -> Model {
        let path = "bench_la02.txt";
        let problem = Problem::read(path).expect("Could not find path");
        let graph = problem.into_graph();

        Model {
            counter: 0,
            problem, graph
        }
    }

    fn update(&mut self, event: Msg) {
        match event {
            Msg::Decrement => { // Calculate the span
                match self.model.graph.critical_path() {
                    Ok((span, _)) => self.model.counter = span,
                    Err(_) => ()
                }
            },
            Msg::Increment => { // Do local search
                use std::time;
                let timer = time::Instant::now();
                let searcher = LocalSearch::new(4000);                
                let graph = searcher.solve(&self.model.problem);
                let end = time::Instant::now();

                println!("Timer: {:?}", end - timer);

                self.model.graph = graph;
                let (span, _) = self.model.graph.critical_path().expect("Graph is not directed for some reason");
                self.model.counter = span;
                
                self.graph.emit(GraphMsg::SetProblem((self.model.problem.clone(), self.model.graph.clone())));
                self.constraints.emit(ConstraintsMsg::SetProblem((self.model.problem.clone(), ProblemConstraints::new(&self.model.graph, span).unwrap())));

            },
            Msg::Fix(a, b) => {
                println!("{:?}->{:?}", a, b);
                let constraints = ProblemConstraints::new(&self.model.graph, UPPER).unwrap();
                let node_1 = &self.model.graph.nodes()[a];
                let node_2 = &self.model.graph.nodes()[b];
                if constraints.check_precedence(node_1, node_2) {
                    let graph = self.model.graph.fix_disjunction(node_1, node_2).expect("not cyclic");
                    let constraints = ProblemConstraints::new(&graph, UPPER).unwrap();

                    self.model.graph = graph;
                    self.graph.emit(GraphMsg::SetProblem((self.model.problem.clone(), self.model.graph.clone())));
                    self.constraints.emit(ConstraintsMsg::SetProblem((self.model.problem.clone(), constraints)));
                } else {
                    println!("Leads to infeasible solution");
                }
            },
            Msg::Quit => gtk::main_quit(),
        }
    }

    view! {
        gtk::Window {
            title: "Jobshop",
            property_default_height: 480,
            property_default_width: 600,
            gtk::Box {
                orientation: Vertical,                
                gtk::Button {
                    // By default, an event with one paramater is assumed.
                    clicked => Msg::Increment,
                    // Hence, the previous line is equivalent to:
                    // clicked(_) => Increment,
                    label: "Do localsearch",
                },
                gtk::Label {
                    // Bind the text property of this Label to the counter attribute
                    // of the model.
                    // Every time the counter attribute is updated, the text property
                    // will be updated too.
                    text: &self.model.counter.to_string(),
                },
                gtk::Button {
                    clicked => Msg::Decrement,
                    label: "Calculate spanning"
                },
                gtk::Box {
                    orientation: Horizontal, 
                    hexpand: true,
                    vexpand: true,
                    gtk::Box {
                        orientation: Vertical,
                        gtk::Label {
                            text: "Enter which edge to fix"
                        },
                        #[name="edge_selection"]
                        EdgeSelection::<LinkedGraph<ProblemNode>>(self.model.graph.clone()) {
                            EdgeFix(a, b) => Fix(a, b),
                        },
                    },
        
                    #[name="graph"]
                    GraphWidget::<LinkedGraph<ProblemNode>>((self.model.problem.clone(), self.model.graph.clone())),
                    #[name="constraints"]
                    ConstraintsWidget(((&self.model.problem).clone(), ProblemConstraints::new(&self.model.graph, UPPER).unwrap()))
                }
            },
            // Use a tuple when you want to both send a message and return a value to
            // the GTK+ callback.
            delete_event(_, _) => (Msg::Quit, Inhibit(false)),
        }
    }
}

fn main() {
    Win::run(()).expect("Could not start window");
}

// fn fix_graph(graph: LinkedGraph<ProblemNode>) -> Result<LinkedGraph<ProblemNode>, disjunctgraph::GraphError> {
//         graph.fix_disjunction(&4, &8)?
//             .fix_disjunction(&1, &7)?
//             .fix_disjunction(&6, &2)?
//             .fix_disjunction(&6, &5)?
//             .fix_disjunction(&2, &5)?
//             .fix_disjunction(&4, &3)?
//             .fix_disjunction(&8, &3)
// }