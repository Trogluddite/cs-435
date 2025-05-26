#[allow(unused_imports)]          //FIXME
use std::{error::Error, result, thread, io};
use log::{info, LevelFilter};

use ratatui::{
    backend::{Backend, CrosstermBackend},
    crossterm::{
        event::{self, Event, KeyCode, KeyEventKind},
        execute,
        terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
    },
    Terminal,
};

//use::lurk_messages::Message;

mod app;
mod ui;

use crate::{
    app::{App, CurrentScreen},
    ui::ui,
};


fn main() -> Result<(), Box<dyn Error>> {
    let _ = simple_logging::log_to_file("./lurkalyzer.log", LevelFilter::Info);

    //set up terminal
    enable_raw_mode()?;
    let mut stderr = io::stderr(); //terminal defaults to stderr/stdout to same stream
    execute!(stderr, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stderr);
    let mut terminal = Terminal::new(backend)?;
    info!("Terminal configured");
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
    info!("Terminal restored");

    if let Err(err) = res{
        println!("{err:?}");
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
                    info!("Pressed C");
                    app.current_screen = CurrentScreen::Configuration;
                    app.set_server(String::from("isoptera.lcsc.edu"), 5005);
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
