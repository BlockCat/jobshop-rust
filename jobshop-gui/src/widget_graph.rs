use std::f64::consts::PI;

use jobshop::problem::{ Problem, ProblemNode };
use disjunctgraph::{ LinkedGraph, Graph, NodeId };

use gdk::RGBA;
use cairo::Context;
use gtk::{
    BoxExt,
    DrawingArea,
    WidgetExt,
};

use relm::{
    DrawHandler,
    Relm,
    Widget,
};

use relm_attributes::widget;

use self::GraphMsg::*;

pub struct Model {
    draw_handler: DrawHandler<DrawingArea>,    
    graph: LinkedGraph<ProblemNode>,
    problem: Problem,
}

#[derive(Msg, Debug)]
pub enum GraphMsg {    
    SetProblem((Problem, LinkedGraph<ProblemNode>)),
    UpdateDrawBuffer,
}

#[widget]
impl Widget for GraphWidget {

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

        //let context = self.model.draw_handler.get_context();
        let allocation = self.drawing_area.get_allocation();

        context.set_source_rgb(1.0, 1.0, 1.0);
        context.paint();                
    
        let problem: &Problem = &self.model.problem;
        let graph: &LinkedGraph<ProblemNode> = &self.model.graph;

        let job_count = problem.jobs.len(); // There will be this many 'lanes'
        let max_activities = problem.jobs.iter().map(|x| x.len()).max().unwrap_or(0); // This is the max length

        let height_buffer = allocation.height as f64 / (job_count + 1) as f64;
        let width_buffer = allocation.width as f64 / (max_activities + 1) as f64;
        let x_offset = width_buffer;
        let y_offset = height_buffer;        
        let circle_size = 15.0;

        // Map activities to locations [(f32, f32)]                    
        struct Node {
            location: (f64, f64),
            label: String,
            colour: RGBA,
        }

        let mut activity_locations = vec!();

        let sourcesink_y = y_offset + ((job_count as f64 - 1.0) / 2.0) * height_buffer;
        let source_x = circle_size;
        let sink_x = x_offset + width_buffer * max_activities as f64 - circle_size;

        activity_locations.push(Node {
            location: (source_x, sourcesink_y),
            label: String::from("S"),
            colour: RGBA { red: 1.0, green: 0.0, blue: 0.01, alpha: 1.0 }
        });
        
        activity_locations.extend(problem.jobs.iter().enumerate().map(|(job_id, activities)| {
            let y = y_offset + job_id as f64 * height_buffer;
            let x_offset = x_offset + (max_activities - activities.len()) as f64 * width_buffer / 2.0;
            activities.iter().enumerate().map(move |(id, x)| {
                let machine = problem.activities[*x].machine_id;
                let colour = match machine {
                    0 => RGBA { red: 0.0, green: 1.0, blue: 0.1, alpha: 1.0 },
                    1 => RGBA { red: 0.3, green: 0.4, blue: 0.1, alpha: 1.0 },
                    2 => RGBA { red: 0.6, green: 0.2, blue: 0.2, alpha: 1.0 },
                    3 => RGBA { red: 0.8, green: 0.0, blue: 0.45, alpha: 1.0 },
                    4 => RGBA { red: 0.1, green: 0.1, blue: 0.0, alpha: 1.0 },
                    5 => RGBA { red: 0.0, green: 0.7, blue: 1.0, alpha: 1.0 },
                    _ => RGBA { red: 0.0, green: 0.0, blue: 0.0, alpha: 1.0 },
                };
                Node {
                    location: (x_offset + id as f64 * width_buffer, y),
                    label: format!("{}", x.id()),
                    colour: colour
                }
                
            })
        }).flatten());
        
        activity_locations.push(Node {
            location: (sink_x, sourcesink_y),
            label: String::from("T"),
            colour: RGBA { red: 1.0, green: 0.0, blue: 0.01, alpha: 1.0 }
        });


        // Draw disjunctions
        context.set_source_rgb(0.1, 0.1, 0.1);
        context.set_antialias(cairo::Antialias::Fast);
        context.set_line_width(1.0);
        context.set_dash(&[1.0], 2.0);
        for activity in graph.nodes().iter() {
            let disjuctions = graph.disjunctions(activity);
            let id = activity.id();

            for other in disjuctions {
                let other = other.id();
                if other > id {
                    let colour = &activity_locations[id].colour;
                    let a = &activity_locations[id].location;
                    let b = &activity_locations[other].location;
                    context.set_source_rgb(colour.red, colour.green, colour.blue);
                    context.move_to(a.0, a.1);
                    context.line_to(b.0, b.1);
                    context.stroke();
                }
            }
        }

        // Draw arcs
        context.set_source_rgb(0.2, 0.2, 0.2);
        context.set_antialias(cairo::Antialias::Fast);
        context.set_line_width(1.8);
        context.set_dash(&[], 0.0);
        for activity in graph.nodes().iter() {
            let successors = graph.successors(activity);                        

            for other in successors {
                let a = &activity_locations[activity.id()].location;
                let b = &activity_locations[other.id()].location;

                let normalized = (b.0 - a.0, b.1 - a.1);
                let length = (normalized.0.powi(2) + normalized.1.powi(2)).sqrt();
                let normalized = (normalized.0 / length, normalized.1 / length);

                let rot_1 = {
                    let angle = 0.7f64;
                    let sin = angle.sin();
                    let cos = angle.cos();

                    (normalized.0 * cos - normalized.1 * sin, normalized.0 * sin + normalized.1 * cos)
                };
                let rot_2 = {
                    let angle = -0.7f64;
                    let sin = angle.sin();
                    let cos = angle.cos();

                    (normalized.0 * cos - normalized.1 * sin, normalized.0 * sin + normalized.1 * cos)
                };
                let b = (b.0 - normalized.0 * circle_size, b.1 - normalized.1 * circle_size);

                //Draw arrow
                context.move_to(b.0 - circle_size * rot_1.0 * 0.4, b.1 - circle_size * rot_1.1 * 0.4);
                context.line_to(b.0, b.1);                
                context.line_to(b.0 - circle_size * rot_2.0 * 0.4, b.1 - circle_size * rot_2.1 * 0.4);
                context.close_path();
                context.fill();

                context.move_to(a.0, a.1);
                context.line_to(b.0, b.1);
                context.stroke();

                //context.rectangle(b.0 - 16.0, b.1 - 16.0, 32.0, 32.0);
                //context.fill();
            }
        }

        // Draw nodes
        context.set_antialias(cairo::Antialias::Good);
        for node in activity_locations.iter() {
            context.set_source_rgb(node.colour.red, node.colour.green, node.colour.blue);
            context.arc(node.location.0, node.location.1, circle_size, 0.0, 2.0 * PI);
            context.fill();

            context.set_source_rgb(1.0, 1.0, 1.0);
            context.move_to(node.location.0 - 3.0, node.location.1);
            context.show_text(&node.label);
        }
    }

    fn update(&mut self, event: GraphMsg) {        
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
            //draw(_, _) => (UpdateDrawBuffer, Inhibit(false)),            
        },
    }
}
