use std::io;
use termion::raw::IntoRawMode;
use termion::screen::AlternateScreen;
use termion::event::Key;

use tui::Terminal;
use tui::backend::TermionBackend;
use tui::widgets::{Widget, Block, Borders};
use tui::layout::{Layout, Constraint, Direction};
use utils::event::{ Event, Events };

use disjunctgraph::{ Graph, LinkedGraph };
use jobshop::problem::*;
use jobshop::schedule::Schedule;

mod utils;

fn main() {

    let stdout = io::stdout().into_raw_mode().unwrap();
    let stdout = AlternateScreen::from(stdout);
    let backend = TermionBackend::new(stdout);
    let mut terminal = Terminal::new(backend).unwrap();
    let events = utils::event::Events::new();

    loop {
        terminal.draw(|mut f| {            
            let chunks = Layout::default().direction(Direction::Vertical)
                .margin(1)
                .constraints(
                [ Constraint::Percentage(10), Constraint::Percentage(80), Constraint::Percentage(10)].as_ref()
                )
                .split(f.size());
            Block::default()
                .title("Block")
                .borders(Borders::ALL)
                .render(&mut f, chunks[0]);
            Block::default()
                .title("Block")
                .borders(Borders::ALL)
                .render(&mut f, chunks[1]);
            Block::default()
                .title("Block")
                .borders(Borders::ALL)
                .render(&mut f, chunks[2]);
        }).unwrap();

        match events.next() {
            Ok(Event::Input(key)) => {
                if key == Key::Char('q') {
                    break;
                }
            }
            _ => {}
        }

    }

    
    /*let path = "bench_test.txt";

    let problem = Problem::read(path).unwrap();
    let graph = ProblemGraph::<LinkedGraph<ProblemNode>>::from(&problem).0;

    //println!("Problem statement: {:#?}", problem);
    //println!("Graph statement: {:?}", graph);    
    let graph = graph.into_directed().unwrap();
    println!("directed graph: {:?}", graph);
    graph.force_critical_path();
    let graph = graph.flip_edge(&5, &6).unwrap();
    graph.force_critical_path();

    let graph = graph.flip_edge(&2, &6).unwrap();
    graph.force_critical_path();

    let graph = graph.flip_edge(&3, &4).unwrap();
    graph.force_critical_path();

    let schedule = Schedule::from_graph(problem, graph);

    println!("Schedule: {:?}", schedule);

    //println!("cyclic graph: {:?}", graph.flip_edge(&3, &9).unwrap());

    /*let mut f = File::create("test.dot").unwrap();
    dot::render(&graph, &mut f).expect("Could not render to vizgraph");*/*/
}