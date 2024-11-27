#[allow(unused_imports)]        //FIXME
use ratatui::{
  layout::{Constraint, Direction, Layout, Rect},
  style::{Color, Style},
  symbols::border,
  text::{Line,Span,Text},
  widgets::{Block, Borders, Clear, List, ListItem, Paragraph, Wrap},
  Frame,
};

#[allow(unused_imports)]        //FIXME
use crate::app::{App, CurrentScreen};

#[allow(unused_variables)] //FIXME
pub fn ui(frame: &mut Frame, app: &App){
    let outer_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage(75),
            Constraint::Percentage(25),
        ])
        .split(frame.area());
    let inner_layout = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(50),
            Constraint::Percentage(50)
        ])
        .split(outer_layout[1]);

    let title = Line::from(" Lurkalyzer ");
    let footer = Line::from(" Press (q) to quit ");
    let block = Block::bordered()
        .title(title.centered())
        .title_bottom(footer.centered())
        .border_set(border::DOUBLE);
    frame.render_widget(block, outer_layout[0]);

    let cfg_title = Line::from(" Config ");
    let incomming_title = Line::from(" Incomming ");
    let cfg_block = Block::bordered()
        .title(cfg_title.centered())
        .border_set(border::DOUBLE);
    frame.render_widget(cfg_block, inner_layout[0]);

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
