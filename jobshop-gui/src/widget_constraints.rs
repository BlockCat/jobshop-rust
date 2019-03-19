use std::f64::consts::PI;

use jobshop::problem::{ Problem, ProblemNode };
use jobshop::schedule::Schedule;
use disjunctgraph::{ LinkedGraph, Graph, NodeId, GraphNode };

use gdk::{EventMask, RGBA};
use cairo::Context;
use gtk::{
    BoxExt,
    DrawingArea,
    Inhibit,
    WidgetExt,
};
use rand::Rng;
use relm::{
    DrawHandler,
    Relm,
    Widget,
    interval,
};

use relm_attributes::widget;

use self::ConstraintsMsg::*;

const SIZE: f64 = 15.0;

pub struct Model {
    draw_handler: DrawHandler<DrawingArea>,    
    graph: LinkedGraph<ProblemNode>,
    problem: Problem,
}

#[derive(Msg, Debug)]
pub enum ConstraintsMsg {
    SetProblem((Problem, LinkedGraph<ProblemNode>)),
    UpdateDrawBuffer,
}

#[widget]
impl Widget for ConstraintsWidget {

    fn init_view(&mut self) {
        self.model.draw_handler.init(&self.drawing_area);

        let context = self.model.draw_handler.get_context();
        self.draw(&context);

    }

    fn model(_: &Relm<Self>, (problem, graph): (Problem, LinkedGraph<ProblemNode>)) -> Model {
        Model {
            draw_handler: DrawHandler::new().expect("draw handler"),            
            graph, problem,
        }
    }

    fn draw(&self, context: &Context) {        
        
        // Graph is drawn from left to right (horizontal)
        context.set_source_rgb(1.0, 1.0, 1.0);
        context.paint();                

        let allocation = self.drawing_area.get_allocation();
    
        let problem: &Problem = &self.model.problem;
        let graph: &LinkedGraph<ProblemNode> = &self.model.graph;
        let schedule = Schedule::from_graph(problem.clone(), graph.clone());
        let max_constraint = 23;

        let y_axis = 15.0;
        let x_axis = 15.0;

        // Draw y-axis
        context.move_to(x_axis, y_axis);
        context.line_to(x_axis, allocation.height as f64);
        context.stroke();
        // Draw x-axis
        context.move_to(x_axis, y_axis);
        context.line_to(allocation.width as f64, y_axis);
        context.stroke();
        
        let height = allocation.height as f64 / (problem.activities.len() + 2) as f64;
        let upper_bound = 1000.0;
        let width = allocation.width as f64 - x_axis;
        let horizontal_scale = width / upper_bound; 

        for (k, activity) in schedule.activities.iter().enumerate() {
            let early_start = activity.starting_time;
            let early_end = early_start + activity.activity.process_time;
            let late_start = early_start + 3;
            let late_end = early_end + 9;
            ConstraintsWidget::draw_bar(x_axis + 5.0, horizontal_scale, height, y_axis + height + k as f64 * (height + 5.0), early_start, early_end, late_start, late_end, context);
        }
    }

    /// Draw a constraint bar
    fn draw_bar(x_offset: f64, horizontal_scale: f64, height: f64, y: f64, early_start: u32, early_end: u32, late_start: u32, late_end: u32, context: &Context) {
        let early_start = early_start as f64 * horizontal_scale;
        let early_end = early_end as f64 * horizontal_scale;
        let late_start = late_start as f64 * horizontal_scale;
        let late_end = late_end as f64 * horizontal_scale;

        context.set_source_rgb(1.0, 0.1, 0.1);
        context.rectangle(x_offset + early_start, y - height * 0.5, early_end - early_start, height);
        context.fill();

        context.set_source_rgb(0.1, 0.1, 1.0);
        context.rectangle(x_offset + late_start, y - height * 0.4, late_end - late_start, height * 0.8);
        context.fill();

        context.set_source_rgb(0.0, 0.0, 0.0);
        context.move_to(x_offset + early_start, y);
        context.line_to(x_offset + late_end, y);
        context.stroke();
    }

    fn update(&mut self, event: ConstraintsMsg) {        
        match event {
            SetProblem((problem, graph)) => {
                self.model.graph = graph;
                self.model.problem = problem;
                let context = self.model.draw_handler.get_context();
                self.draw(&context);
            }, 
            UpdateDrawBuffer => {
                let context = self.model.draw_handler.get_context();
                self.draw(&context);
            },
        }
    }

    view! {
        #[name="drawing_area"]
        gtk::DrawingArea {                
            child: {                    
                expand: true,
            },
            configure_event(_, _) => (UpdateDrawBuffer, false),            
        },
    }
}
