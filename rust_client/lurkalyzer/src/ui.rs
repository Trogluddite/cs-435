use ratatui::{
  layout::{Constraint, Direction, Layout, Rect},
  style::{Color, Style},
  text::{Line,Span,Text},
  widgets::{Block, Borders, Clear, List, ListItem, Paragraph, Wrap},
  Frame,
};

use crate::app::{App, CurrentScreen};

pub fn ui(frame: &mut Frame, app: &app){
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),
            Constraint::Min(1),
            Constraint::Length(3),
        ])
        .split(frame.area());
    let title_block = Block::default()
        .borders(Borders::ALL)
        .style(Style::default());
    let title = Paragraph::new(Text::styled(
        "Lurkalyzer",
        Style::default().fg(Color::Green),
    ))
    .block(title_block);
}

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
            Constraint::Percentage((100 - percentage_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
}
