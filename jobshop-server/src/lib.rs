
#![feature(proc_macro_hygiene, decl_macro)]

#[macro_use] extern crate rocket;
#[macro_use] extern crate serde_derive;

extern crate serde;

extern crate rocket_contrib;
extern crate disjunctgraph;
extern crate jobshop;

extern crate clap;

use rocket::{ Rocket, State };
use rocket_contrib::json::Json;
use rocket_contrib::templates::Template;

use jobshop::problem::{ ProblemNode, Problem };

pub type LinkedGraph = disjunctgraph::LinkedGraph<ProblemNode>; 
type GraphState = std::sync::Mutex<std::cell::RefCell<Option<ProgramState>>>;

const graph_paths: [&'static str; 2] = ["bench_test", "bench_la02"];

pub fn create_rocket() -> Rocket {
    rocket::ignite()
        .attach(Template::fairing())
        .manage(GraphState::default())
    .mount("/", routes![index, synchronize, execute_command])
}

#[get("/")]
fn index() -> Template {
    let context = std::collections::HashMap::<String, String>::new();
    Template::render("index", context)
}

#[get("/exec/<command>")]
fn execute_command(command: String, state: State<GraphState>) -> rocket::http::Status {
    use clap::{Arg, App, SubCommand };

    let mut program = vec!("jobshop-server");
    program.extend(command.split(" "));

    let matches = App::new("jobshop-server")        
        .version("1.0")
        .author("Zino Onomiwo")
        .about("The server for executing things")
        .subcommand(App::new("reset")
            .about("Reset the state")
        )
        .subcommand(App::new("load")
            .about("Load state")
            .arg(Arg::with_name("INPUT")                
                .required(true)                
            )
        )
        .get_matches_from(program);

    if let Some(matches) = matches.subcommand_matches("reset") {
        return reset(state);
    }   

    if let Some(matches) = matches.subcommand_matches("load") {
        let index = matches.value_of("INPUT").unwrap();
        if let Ok(index) = index.parse::<usize>() {
            return load(index, state);
        } else {
            return rocket::http::Status::new(400, "Invalid problem index")
        }
    }


    rocket::http::Status::new(404, "Command not found")
            
}

fn reset(state: State<GraphState>) -> rocket::http::Status {
    let m = state.inner().lock().unwrap();
    
    *(m.borrow_mut()) = None;    

    rocket::http::Status::new(202, "Program state reset")
}

fn load(index: usize, state: State<GraphState>) -> rocket::http::Status {
    if index >= graph_paths.len() {
        return rocket::http::Status::new(400, "Problem does not exist");
    }

    
    let problem = Problem::read(format!("{}.txt", graph_paths[index])).expect("Could not read problem");
    let graph = problem.into_graph::<LinkedGraph>();

    let program_state = ProgramState { problem, graph };
    
    *(state.inner().lock().unwrap().borrow_mut()) = Some(program_state);    

    rocket::http::Status::new(202, "Accepted and loaded problem")
}


#[derive(Serialize)]
struct SynchronizedState {

}

#[derive(Serialize)]
struct WrongState;

/// Send the graph and problem and the entire state.
#[get("/sync")]
fn synchronize(state: State<GraphState>) -> Result<Json<SynchronizedState>, rocket::http::Status> {
    
    if let None = *state.inner().lock().unwrap().borrow() {
        return Result::Err(rocket::http::Status::new(400, "No state is yet defined"));
    }

    let sync = SynchronizedState {};


    Ok(Json(sync))
}

struct ProgramState {
    problem: Problem,
    graph: LinkedGraph,
}