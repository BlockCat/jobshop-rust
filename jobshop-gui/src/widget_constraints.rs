use jobshop::problem::Problem;
use jobshop::constraints::*;

use cairo::Context;
use gtk::prelude::*;
use gtk::DrawingArea;    

use relm::{
    DrawHandler,
    Relm,
    Widget,
};

use relm_attributes::widget;

use self::ConstraintsMsg::*;


pub struct Model {
    draw_handler: DrawHandler<DrawingArea>,    
    constraints: ProblemConstraints,
    problem: Problem,
}

#[derive(Msg, Debug)]
pub enum ConstraintsMsg {
    SetProblem((Problem, ProblemConstraints)),
    UpdateDrawBuffer,
}

#[widget]
impl Widget for ConstraintsWidget {

    fn init_view(&mut self) {
        self.model.draw_handler.init(&self.drawing_area);

        let context = self.model.draw_handler.get_context();
        self.draw(&context);

    }

    fn model(_: &Relm<Self>, (problem, constraints): (Problem, ProblemConstraints)) -> Model {
        Model {
            draw_handler: DrawHandler::new().expect("draw handler"),            
            constraints, problem,
        }
    }

    fn draw(&self, context: &Context) {
        use cairo::PatternTrait;
        // Graph is drawn from left to right (horizontal)
        context.set_source_rgb(1.0, 1.0, 1.0);
        context.paint();                

        let allocation = self.drawing_area.get_allocation();
    
        let problem: &Problem = &self.model.problem;
        let constraints: &ProblemConstraints = &self.model.constraints;        

        let upper_bound = constraints.upper_bound as f64;
        let y_axis = 15.0;
        let x_axis = 15.0;

        let height = 10.0;
        
        let width = allocation.width as f64 - x_axis;
        let horizontal_scale = width / upper_bound; 

        // Draw y-axis
        context.set_source_rgb(0.0, 0.0, 0.0);
        context.set_line_width(2.0);
        context.move_to(x_axis, y_axis);
        context.line_to(x_axis, allocation.height as f64);
        context.stroke();

        context.set_source_rgb(0.5, 0.5, 0.5);
        context.set_line_width(0.2);
        //context.set_dash(&[2.0], 2.0);
        /*let lines = 250;
        for i in 0..(lines) as u32 {
            let x_axis = 5.0 + x_axis + horizontal_scale * i as f64;
            context.move_to(x_axis, y_axis);
            context.line_to(x_axis, allocation.height as f64);
            context.stroke();
        }*/

        context.set_source_rgb(0.0, 0.0, 0.0);
        context.set_line_width(2.0);
        context.set_dash(&[], 0.0);
        // Draw x-axis
        context.move_to(x_axis, y_axis);
        context.line_to(allocation.width as f64, y_axis);
        context.stroke();
        
        let pattern_1 = {
            let surface = cairo::ImageSurface::create(cairo::Format::ARgb32, 4, 4).unwrap();
            let context = Context::new(&surface);

            context.set_source_rgb(0.0, 0.0, 0.0);
            context.set_line_width(1.0);
            context.move_to(0.0, 0.0);
            context.line_to(4.0, 4.0);
            context.stroke();

            let pattern = cairo::Pattern::SurfacePattern(cairo::SurfacePattern::create(&surface));
            pattern.set_extend(cairo::Extend::Repeat);

            pattern
        };

        let pattern_2 = {
            let surface = cairo::ImageSurface::create(cairo::Format::ARgb32, 4, 4).unwrap();
            let context = Context::new(&surface);

            context.set_source_rgb(0.0, 0.0, 0.0);
            context.set_line_width(1.0);
            context.move_to(4.0, 0.0);
            context.line_to(0.0, 4.0);
            context.stroke();

            let pattern = cairo::Pattern::SurfacePattern(cairo::SurfacePattern::create(&surface));
            pattern.set_extend(cairo::Extend::Repeat);

            pattern
        };
        for (k, activity) in problem.activities.iter().enumerate() {
            let constraint = &constraints.constraints[k + 1];
            let early_start = constraint.left_bound;
            let early_end = early_start + activity.process_time;
            let late_end = constraint.right_bound;
            let late_start = late_end - activity.process_time;
            
            ConstraintsWidget::draw_bar(x_axis + 5.0, horizontal_scale, height, y_axis + height + k as f64 * (height + 0.0), early_start, early_end, late_start, late_end, &pattern_1, &pattern_2, context);
        }
    }

    /// Draw a constraint bar
    fn draw_bar(x_offset: f64, horizontal_scale: f64, height: f64, y: f64, early_start: u32, early_end: u32, late_start: u32, late_end: u32, pattern_1: &cairo::Pattern, pattern_2: &cairo::Pattern, context: &Context) {
        
        let early_start = early_start as f64 * horizontal_scale;
        let early_end = early_end as f64 * horizontal_scale;
        let late_start = late_start as f64 * horizontal_scale;
        let late_end = late_end as f64 * horizontal_scale;

        context.set_antialias(cairo::Antialias::Gray);

        // Draw earliest box
        // -------------------------------
        // draw pattern        
        context.set_source(&pattern_1);        
        context.rectangle(x_offset + early_start, y - height * 0.5, early_end - early_start, height);
        context.fill();

        // draw box
        context.set_source_rgb(0.0, 0.0, 0.0);
        context.set_line_width(1.0);
        context.rectangle(x_offset + early_start, y - height * 0.5, early_end - early_start, height);
        context.stroke();

        // Draw latest box
        // -------------------------------
        // draw pattern
        context.set_source(&pattern_2);
        context.rectangle(x_offset + late_start, y - height * 0.5, late_end - late_start, height);
        context.fill();

        // draw box
        context.set_source_rgb(0.0, 0.0, 0.0);
        context.set_line_width(1.0);
        context.rectangle(x_offset + late_start, y - height * 0.5, late_end - late_start, height);
        context.stroke();
        // -------------------------------

        context.set_source_rgb(0.0, 0.0, 0.0);
        context.move_to(x_offset + early_start, y);
        context.line_to(x_offset + late_end, y);
        context.stroke();
    }

    fn update(&mut self, event: ConstraintsMsg) {        
        match event {
            SetProblem((problem, constraints)) => {
                self.model.constraints = constraints;
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
