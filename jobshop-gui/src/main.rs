extern crate gtk;
#[macro_use]
extern crate relm;
#[macro_use]
extern crate relm_derive;
#[macro_use]
extern crate relm_attributes;

use relm_attributes::widget;
use relm::DrawHandler;

use relm::{Relm, Update, Widget};
use gtk::prelude::*;
use gtk::{Window, Inhibit, WindowType};
use gtk::Orientation::Vertical;

use disjunctgraph::{ Graph, LinkedGraph };
use jobshop::problem::*;
use jobshop::schedule::Schedule;


#[derive(Msg)]
pub enum Msg {
    // …
    Quit,
}

pub struct Model {
    // …
}

#[widget]
impl Widget for Win {   

    fn model() -> Model {
        Model {}
    }

    fn update(&mut self, event: Msg) {
        
    }

    // Create the widgets.
    widget! {
        gtk::Window {
            property_default_height: 650,
            property_default_width: 1000,
            title: "Window title",
            gtk::Box {
                orientation: Vertical,               
                gtk::Label {
                    // Bind the text property of this Label to the counter attribute
                    // of the model.
                    // Every time the counter attribute is updated, the text property
                    // will be updated too.
                    text: "Hello world",
                },
            },
        }
    }
    /*fn view(relm: &Relm<Self>, model: Self::Model) -> Self {
        use gtk::Orientation::Vertical;
        use gtk::{ Button, Label };
        // GTK+ widgets are used normally within a `Widget`.
        let window = Window::new(WindowType::Toplevel);
        

        let vbox = gtk::Box::new(Vertical, 0);

        let counter_label = Label::new("Hello world");
        vbox.add(&counter_label);
        let counter_label = Label::new("Hello world 2");
        vbox.add(&counter_label);
        let counter_label = Label::new("Hello world 3");
        vbox.add(&counter_label);

        // Connect the signal `delete_event` to send the `Quit` message.
        connect!(relm, window, connect_delete_event(_, _), return (Some(Msg::Quit), Inhibit(false)));
        // There is also a `connect!()` macro for GTK+ events that do not need a
        // value to be returned in the callback.

        window.add(&vbox);
        window.show_all();

        Win {
            model,
            window: window,
        }
    }*/
}

fn main() {
    Win::run(()).unwrap();
}

/*
fn main() {
    
    let path = "bench_test.txt";

    let problem = Problem::read(path).unwrap();
    let graph = ProblemGraph::<LinkedGraph<ProblemNode>>::from(&problem).0;

    //println!("Problem statement: {:#?}", problem);
    //println!("Graph statement: {:?}", graph);    
    
    let graph = fix_graph(graph).unwrap();
    let (span, _) = graph.force_critical_path();
    let schedule = Schedule::from_graph(problem, graph);

    println!("Schedule: {:#?}", schedule);
    println!("with span: {}", span);

    //println!("cyclic graph: {:?}", graph.flip_edge(&3, &9).unwrap());

    /*let mut f = File::create("test.dot").unwrap();
    dot::render(&graph, &mut f).expect("Could not render to vizgraph");*/
}*/

fn fix_graph(graph: LinkedGraph<ProblemNode>) -> Result<LinkedGraph<ProblemNode>, disjunctgraph::GraphError> {
        graph.fix_disjunction(&4, &8)?
            .fix_disjunction(&1, &7)?
            .fix_disjunction(&6, &2)?
            .fix_disjunction(&6, &5)?
            .fix_disjunction(&2, &5)?
            .fix_disjunction(&4, &3)?
            .fix_disjunction(&8, &3)
}