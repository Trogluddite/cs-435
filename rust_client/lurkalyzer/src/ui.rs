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
    let layout_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage(100),
        ])
        .split(frame.area());

    let title = Line::from(" Lurkalyzer ");
    let footer = Line::from(" Press (q) to quit ");

    let block = Block::bordered()
        .title(title.centered())
        .title_bottom(footer.centered())
        .border_set(border::DOUBLE);

    frame.render_widget(block, layout_chunks[0]);
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
