use disjunctgraph::{ Graph, GraphNode, NodeId, LinkedGraph };
use jobshop::problem::ProblemNode;

use relm::{ Update, Relm, Widget };
use gtk::prelude::*;

pub struct EdgeSelection {
    model: EdgeModel,
    root: gtk::Box,
}

pub struct EdgeModel {
    graph: LinkedGraph<ProblemNode>,
}

#[derive(Debug, Msg)]
pub enum EdgeMsg {
    Fix(usize, usize),
    Unfix(usize, usize),
    Swap(usize, usize),
}

impl Update for EdgeSelection {
    type Model = EdgeModel;
    type ModelParam = LinkedGraph<ProblemNode>;
    type Msg = EdgeMsg;

    fn model(_: &Relm<Self>, graph: Self::ModelParam) -> Self::Model {
        EdgeModel {
            graph
        }
    }

    fn update(&mut self, event: Self::Msg) {

    }
}

impl Widget for EdgeSelection {
    type Root = gtk::Box;

    fn root(&self) -> Self::Root {
        self.root.clone()
    }

    fn view(relm: &Relm<Self>, model: Self::Model) -> Self {
        let container = gtk::Box::new(gtk::Orientation::Vertical, 1);

        let graph: &LinkedGraph<ProblemNode> = &model.graph;
        let disjunctions = graph.nodes()
            .iter()
            .map(|x| x.id())
            .map(|id| graph.disjunctions(&id).into_iter().map(move |x| (id, x.id())) )
            .flatten()
            .filter(|(a, b)| a > b );

        for (a, b) in disjunctions {
            let label = format!("{} -> {}", a, b);
            let button = gtk::Button::new();
            button.set_label(&label);
            container.add(&button);

            
            connect!(relm, button, connect_clicked(_), Some(EdgeMsg::Fix(a, b)));

            let label = format!("{} -> {}", b, a);
            let button: gtk::Button = gtk::Button::new();
            button.set_label(&label);
            container.add(&button);

            connect!(relm, button, connect_clicked(_), Some(EdgeMsg::Fix(b, a)));
        }

        container.show_all();

        EdgeSelection {
            model, 
            root: container,
        }

    }
}