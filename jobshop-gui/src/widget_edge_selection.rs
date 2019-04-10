use disjunctgraph::{ Graph, NodeId };

use relm::{ Update, Relm, Widget };
use gtk::prelude::*;

pub struct EdgeSelection<I: Graph> {
    model: EdgeModel<I>,
    root: gtk::Box,
}

pub struct EdgeModel<I: Graph> {
    graph: I
}

#[derive(Debug, Msg)]
pub enum EdgeMsg {
    Fix(usize, usize),
    Unfix(usize, usize),
    Swap(usize, usize),
}

impl<I: Graph> Update for EdgeSelection<I> {
    type Model = EdgeModel<I>;
    type ModelParam = I;
    type Msg = EdgeMsg;

    fn model(_: &Relm<Self>, graph: Self::ModelParam) -> Self::Model {
        EdgeModel {
            graph,            
        }
    }

    fn update(&mut self, _event: Self::Msg) {

    }
}

impl<I: Graph> Widget for EdgeSelection<I> {
    type Root = gtk::Box;

    fn root(&self) -> Self::Root {
        self.root.clone()
    }

    fn view(relm: &Relm<Self>, model: Self::Model) -> Self {
        let scroll_container = gtk::Box::new(gtk::Orientation::Vertical, 1);
        let container = gtk::Box::new(gtk::Orientation::Vertical, 1);

        container.set_vexpand(true);
        scroll_container.set_vexpand(true);

        
        let scrollbar = gtk::ScrolledWindow::new(None, None);

        let graph: &I = &model.graph;
        let disjunctions = graph.nodes()
            .iter()
            .map(|x| x.id())
            .map(|id| graph.disjunctions(&id).into_iter().map(move |x| (id, x.id())) )
            .flatten()
            .filter(|(a, b)| a > b );

        for (a, b) in disjunctions {

            let mini_box = gtk::Box::new(gtk::Orientation::Horizontal, 0);
            let label = format!("{} -> {}", a, b);
            let button = gtk::Button::new();
            button.set_label(&label);
            mini_box.add(&button);

            
            connect!(relm, button, connect_clicked(_), Some(EdgeMsg::Fix(a, b)));

            let label = format!("{} -> {}", b, a);
            let button: gtk::Button = gtk::Button::new();
            button.set_label(&label);
            mini_box.add(&button);

            connect!(relm, button, connect_clicked(_), Some(EdgeMsg::Fix(b, a)));

            container.add(&mini_box);
        }

        scrollbar.add(&container);
        scroll_container.add(&scrollbar);
        scrollbar.show_all();
        container.show_all();
        scroll_container.show_all();


        EdgeSelection {
            model, 
            root: scroll_container,
        }

    }
}