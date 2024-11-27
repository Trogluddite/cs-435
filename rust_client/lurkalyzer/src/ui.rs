#[allow(unused_imports)]        //FIXME
use ratatui::{
  layout::{Constraint, Direction, Layout, Rect},
  style::{Color, Style, Stylize},
  symbols::border,
  text::{Line,Span,Text},
  widgets::{Block, Borders, Clear, List, ListItem, Table, Row, Paragraph, Wrap},
  Frame,
};

#[allow(unused_imports)]        //FIXME
use crate::app::{App, CurrentScreen};

#[allow(unused_variables)] //FIXME
pub fn ui(frame: &mut Frame, app: &App){
    let outer_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),
            Constraint::Percentage(100),
        ])
        .split(frame.area());
    let inner_layout = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(50),
            Constraint::Percentage(50),
        ])
        .split(outer_layout[1]);

    let title = Line::from(" Lurkalyzer ");
    let footer = Line::from(" Press (q) to quit ");
    let header_block = Block::bordered()
        .title(title.centered())
        .title_bottom(footer.centered())
        .border_set(border::DOUBLE);

    let line: Line = vec![
        "mode".blue(),
        ": ".into(),
        "raw ".red().bold().into(),
        " | ".into(),
        "endiannes: ".blue(),
        "little ".red().bold().into(),
        " | ".into(),
        "send type".blue(),
        ": ".into(),
        "immediate ".red().bold().into(),
    ].into();
    frame.render_widget(Paragraph::new(line).block(header_block), outer_layout[0]);

    let outgoing_title = Line::from(" Outgoing ");
    let incomming_title = Line::from(" Incomming ");
    let outgoing_block = Block::bordered()
        .title(outgoing_title.centered())
        .border_set(border::DOUBLE);
    frame.render_widget(outgoing_block, inner_layout[0]);

    let incomming_block = Block::bordered()
        .title(incomming_title.centered())
        .border_set(border::DOUBLE);
    frame.render_widget(incomming_block, inner_layout[1]);
}

#[allow(dead_code)]
fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y) / 2),
        ])
        .split(r);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ]).split(popup_layout[1])[1]
}
