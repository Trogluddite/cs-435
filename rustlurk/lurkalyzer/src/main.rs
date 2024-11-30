#[allow(unused_imports)]          //FIXME
use std::{error::Error, result, thread, io};
use std::net::{TcpListener, TcpStream, Shutdown};
use std::sync::mpsc::{channel, Receiver, Sender};
use std::sync::{Arc, Mutex};

use ratatui::{
    backend::{Backend, CrosstermBackend},
    crossterm::{
        event::{self, Event, KeyCode, KeyEventKind},
        execute,
        terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
    },
    Terminal,
};

use::lurk_messages::{Message};

mod app;
mod ui;

use crate::{
    app::{App, CurrentScreen},
    ui::ui,
};


fn main() -> Result<(), Box<dyn Error>> {
    //set up terminal
    enable_raw_mode()?;
    let mut stderr = io::stderr(); //terminal defaults to stderr/stdout to same stream
    execute!(stderr, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stderr);
    let mut terminal = Terminal::new(backend)?;
    //create app & run it
    let mut app = App::new();
    let res = run_app(&mut terminal, &mut app);

    //restore terminal
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
    )?;
    terminal.show_cursor()?;

    if let Err(err) = res{
        println!("{err:?}");
    }

    // set up network
    let ip_addr = String::from("127.0.0.1"); //change with ui config
    let port = 5005;
    let address = format!("{}:{}", ip_addr, port.to_string());

    let listener = TcpListener::bind(&address).unwrap();

    let (sender, receiver) = channel();
    let reciever = Arc::new(Mutex::new(receiver));
    thread::spawn(move || handle_mpsc_thread_messages(reciever));
    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                let stream = Arc::new(stream);
                let sender = sender.clone();
                thread::spawn(move || handle_client(stream, sender));
            }
            Err(e) => {
                println!("Error was {}", e);
            }
        }
    }
    Ok(())
}

#[allow(unused_variables)]   //FIXME
fn run_app<B: Backend>(terminal: &mut Terminal<B>, app: &mut App) -> io::Result<bool> {
    loop {
        terminal.draw(|frame| ui(frame, app))?;
        if let Event::Key(key) = event::read()? {
            if key.kind == KeyEventKind::Release{
                // we only care about KeyEventKind::press
                continue;
            }
            match key.code {
                KeyCode::Char('m') => {
                    app.current_screen = CurrentScreen::Main;
                }
                KeyCode::Char('c') => {
                    app.current_screen = CurrentScreen::Configuration;
                }
                KeyCode::Char('r') => {
                    app.current_screen = CurrentScreen::RawMode;
                }
                KeyCode::Char('l') => {
                    app.current_screen = CurrentScreen::LurkMode;
                }
                KeyCode::Char('q') => {
                    return Ok(true);
                }
                // add exit screen
                // add type/edit handlers
                _ => {}
            }
        }
    }
}

#[allow(dead_code)]
fn handle_client(stream: Arc<TcpStream>, sender: Sender<Message>) -> Result<(), Box<dyn Error + Send + Sync>> {
    println!("do nothing right now");

    Ok(())
}

fn handle_mpsc_thread_messages(reciever : Arc<Mutex<Receiver<Message>>>) -> Result<(), Box<dyn Error + Send + Sync>> {
    loop{
        let rec = reciever.lock();
        let message = rec
            .unwrap()
            .recv()
            .map_err( |e| {
                println!("Got error {}", e);
            });

        match message {
            _ => {
                println!("fat arrow");
            }
        }
    }
}
