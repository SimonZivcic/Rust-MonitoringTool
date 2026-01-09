use ratatui::{
    style::{Color, Style},
    widgets::{Block, Borders, Cell, Row, Table},
    layout::Constraint,
};
use Monitor_Lib::models::Server;

pub fn draw_server_table(data: &[(Server, i32, f32, f32)]) -> Table {
    let header = Row::new(vec![
        Cell::from("ID"),
        Cell::from("STAV"),
        Cell::from("NAZOV"),
        Cell::from("ODOZVA"),
        Cell::from("RAM USAGE"),
        Cell::from("CPU %"),
    ]).style(Style::default().fg(Color::Yellow)).bottom_margin(1);

    let rows = data.iter().map(|(s, ms, cpu, ram)| {
        let status_style = match s.status.as_str() {
            "ON" => Style::default().fg(Color::Green),
            "OFF" => Style::default().fg(Color::Red),
            _ => Style::default().fg(Color::DarkGray), // Stav "/"
        };

        Row::new(vec![
            Cell::from(s.id.to_string()),
            Cell::from(s.status.clone()).style(status_style),
            Cell::from(s.name.clone()),
            Cell::from(format!("{}ms", ms)),
            Cell::from(format!("{:.2} / {:.1}GB", ram, s.max_ram)),
            Cell::from(format!("{:.1}%", cpu)),
        ])
    });

    Table::new(rows, [
        Constraint::Length(4),
        Constraint::Length(8),
        Constraint::Percentage(30),
        Constraint::Length(10),
        Constraint::Length(20),
        Constraint::Length(10),
    ])
    .header(header)
    .block(Block::default().borders(Borders::ALL).title(" MONITORING - Stlac 'q' pre koniec "))
}