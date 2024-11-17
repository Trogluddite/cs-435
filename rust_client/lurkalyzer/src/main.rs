use std::{error::Error, io};

use ratatui::{
    backend::{Backend, CrosstermBackend},
    crossterm::{
        event::{self, Event, KeyCode, KeyEventKind},
        execute,
        terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
    },
    Terminal,
};

mod app;
//mod ui;

use crate::{
    app::{App, Screen},
    //ui::ui,
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

    Ok(())
}

fn run_app<B: Backend>(terminal: &mut Terminal<B>, app: &mut App) -> io::Result<bool> {
    loop {
        //terminal.draw(|frame| ui(frame, app))?;
        if let Event::Key(key) = event::read()? {
            if key.kind == KeyEventKind::Release{
                // we only care about KeyEventKind::press
                continue;
            }
            match key.code {
                KeyCode::Char('m') => {
                    app.current_screen = Screen::Main;
                }
                KeyCode::Char('c') => {
                    app.current_screen = Screen::Connfiguration;
                }
                KeyCode::Char('r') => {
                    app.current_screen = Screen::RawMode;
                }
                KeyCode::Char('l') => {
                    app.current_screen = Screen::LurkMode;
                }
                // add exit screen
                // add type/edit handlers
                _ => {}
            }
        }
    }
}
