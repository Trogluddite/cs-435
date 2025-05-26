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
use lurk_messages::MessageType;

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
    let inner_outgoing_line = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(5),
        ])
        .split(inner_layout[0]);
    let inner_outgoing_layout = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Max(6),
            Constraint::Max(6),
        ])
        .split(inner_outgoing_line[0]);


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

    let inner_block1 = Block::bordered()
        .title(Line::from("0").left_aligned())
        .border_set(border::EMPTY);
    let inner_block2 = Block::bordered()
        .title(Line::from("1").left_aligned())
        .border_set(border::EMPTY);

    let outgoing_block = Block::bordered()
        .title(outgoing_title.centered())
        .border_set(border::DOUBLE);
    let inner1 = outgoing_block.inner(inner_outgoing_layout[0]);
    let inner2 = outgoing_block.inner(inner_outgoing_layout[1]);
    frame.render_widget(outgoing_block, inner_layout[0]);

    let byte_line1: Line = vec![
        "BE".green().into(),
    ].into();
    let byte_line2: Line = vec![
        "AD".blue().into(),
    ].into();
    frame.render_widget(Paragraph::new(byte_line1).block(inner_block1), inner1);
    frame.render_widget(Paragraph::new(byte_line2).block(inner_block2), inner2);


    let list_area = message_type_list(frame.area());
    let incomming_block = Block::bordered()
        .title(incomming_title.centered())
        .border_set(border::DOUBLE);
    //frame.render_widget(incomming_block, list_area/*inner_layout[1]*/);
    frame.render_widget(incomming_block, inner_layout[1]);
}

fn message_type_list(area: Rect) -> Rect {
    let mut items: Vec<String> = vec![];
    items.push(String::from("Accept"));
    items.push(String::from("Changeroom"));
    items.push(String::from("Character"));
    items.push(String::from("Connection"));
    items.push(String::from("Error"));
    items.push(String::from("Fight"));
    items.push(String::from("Game"));
    items.push(String::from("Leave"));
    items.push(String::from("Loot"));
    items.push(String::from("Message"));
    items.push(String::from("Room"));
    items.push(String::from("Start"));
    items.push(String::from("PVPFight"));
    items.push(String::from("Version"));

    let layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Fill(1),
            Constraint::Fill(1),
    ]).split(area);

    Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Fill(1),
            Constraint::Fill(1),
        ]).split(layout[1])[1]
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
