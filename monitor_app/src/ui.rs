use ratatui::{
    layout::{Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    widgets::{Block, Borders, List, ListItem, Paragraph, Row, Table, TableState},
    Frame,
};
use Monitor_Lib::models::Server;
use chrono::Utc;
use crate::{ActiveBlock, InfoMode};

pub fn draw_main_layout(
    f: &mut Frame,
    data: &[(Server, i32, f32, f32)],
    state: &mut TableState,
    app_state: &crate::AppState,
) {
    // Rozdelenie
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage(67),
            Constraint::Percentage(30),
            Constraint::Length(1), 
        ])
        .split(f.area());

    let top_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(70), Constraint::Percentage(30)])
        .split(chunks[0]);

    let server_style = if app_state.active_block == ActiveBlock::Servers { Style::default().fg(Color::Yellow) } else { Style::default().fg(Color::White) };
    let info_style = if app_state.active_block == ActiveBlock::Info { Style::default().fg(Color::Yellow) } else { Style::default().fg(Color::White) };

    //NÁPOVEDA V TITULKOCH
    let server_title = if app_state.active_block == ActiveBlock::Servers {
        " SERVERY | [ENTER] ON/OFF | [A] Activate | [R] Remove "
    } else { " SERVERY " };

    let info_title = match app_state.info_mode {
        InfoMode::DeleteConfirm => " Zmazať? | [ENTER] Áno | [ESC] Nie ",
        InfoMode::ConfirmWarning => " CHYBA | [ENTER] Pokračovať | [ESC] Späť ",
        InfoMode::View if app_state.active_block == ActiveBlock::Info => " INFO | [N] Nový | [U] Upraviť ",
        InfoMode::View => " INFO ",
        _ if app_state.update_id.is_some() => " UPRAVIŤ | [ENTER] Ďalej | [ESC] Zrušiť",
        _ => " PRIDAŤ | [ENTER] Ďalej | [ESC] Zrušiť ",
    };

    //TABUĽKA SERVEROV
    let rows = data.iter().map(|(s, ms, cpu, ram)| {
        let is_transitioning = s.status == "Starting" || s.status == "Stopping";
        let style = match s.status.as_str() {
            "ON" => Style::default().fg(Color::Green),
            "OFF" => Style::default().fg(Color::Red),
            "Starting" | "Stopping" => Style::default().fg(Color::Yellow).add_modifier(Modifier::ITALIC),
            _ => Style::default().fg(Color::DarkGray),
        };

        let d_ms = if s.status == "ON" && *ms != -1 && !is_transitioning { format!("{}ms", ms) } else { "0ms".into() };
        let (d_ram, d_cpu) = if is_transitioning || s.status == "OFF" {
            ("0.0/0.0G".into(), "0.0%".into())
        } else {
            (format!("{:.1}/{:.1}G", ram, s.max_ram), format!("{:.1}%", cpu))
        };

        Row::new(vec![s.id.to_string(), s.status.clone(), s.name.clone(), d_ms, d_ram, d_cpu]).style(style)
    });

    let table = Table::new(rows, [Constraint::Length(3), Constraint::Length(9), Constraint::Percentage(30), Constraint::Length(8), Constraint::Length(12), Constraint::Length(8)])
        .header(Row::new(vec!["ID", "STAV", "NÁZOV", "ODOZVA", "RAM", "CPU %"]).style(Style::default().fg(Color::Yellow)))
        .block(Block::default().borders(Borders::ALL).title(server_title).border_style(server_style))
        .row_highlight_style(Style::default().add_modifier(Modifier::REVERSED).fg(Color::Cyan));

    f.render_stateful_widget(table, top_chunks[0], state);

    //INFO PANEL
    match app_state.info_mode {
        InfoMode::View => {
            let mut text = "\n Vyber server...".to_string();
            if let Some(idx) = state.selected() {
                if let Some((s, _, _, _)) = data.get(idx) {
                    let rt = if let Some(st) = app_state.start_times.get(&s.id) {
                        let d = Utc::now().signed_duration_since(*st);
                        format!("{}m {}s", d.num_minutes(), d.num_seconds() % 60)
                    } else { "Offline".into() };
                    text = format!("\n Port:     {}\n CPU:      {}\n Status:   {}\n\n RUN TIME: {}", s.port, s.cpu_model, s.status, rt);
                }
            }
            f.render_widget(Paragraph::new(text).block(Block::default().borders(Borders::ALL).title(info_title).border_style(info_style)), top_chunks[1]);
        }
        InfoMode::DeleteConfirm => {
            let text = format!("\n Naozaj zmazať:\n {}?", app_state.new_name);
            f.render_widget(Paragraph::new(text).style(Style::default().fg(Color::Red)).block(Block::default().borders(Borders::ALL).title(info_title).border_style(Style::default().fg(Color::Red))), top_chunks[1]);
        }
        _ => {
            let sel = Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD);
            let items = vec![
                ListItem::new(format!(" Názov:  {}", app_state.new_name)).style(if matches!(app_state.info_mode, InfoMode::AddServerName | InfoMode::UpdateServerName) { sel } else { Style::default() }),
                ListItem::new(format!(" Port:   {}", app_state.new_port)).style(if matches!(app_state.info_mode, InfoMode::AddServerPort | InfoMode::UpdateServerPort) { sel } else { Style::default() }),
                ListItem::new(format!(" Max Ram:{}", app_state.new_ram)).style(if matches!(app_state.info_mode, InfoMode::AddServerRam | InfoMode::UpdateServerRam) { sel } else { Style::default() }),
                ListItem::new(format!(" CPU:    {}", app_state.new_cpu)).style(if matches!(app_state.info_mode, InfoMode::AddServerCpu | InfoMode::UpdateServerCpu) { sel } else { Style::default() }),
            ];
            f.render_widget(List::new(items).block(Block::default().borders(Borders::ALL).title(info_title).border_style(info_style)), top_chunks[1]);
        }
    }

    //LOGS
    let logs: Vec<ListItem> = app_state.logs.iter().rev()
        .map(|l| {
            let s = if l.contains("ERROR") { Style::default().fg(Color::Red) }
                    else if l.contains("Starting") || l.contains("Stopping") { Style::default().fg(Color::Cyan) }
                    else { Style::default() };
            ListItem::new(l.as_str()).style(s)
        }).collect();
    f.render_widget(List::new(logs).block(Block::default().borders(Borders::ALL).title(" LOGS ")), chunks[1]);
    
    //NÁPOVEDA
    let help_menu = Paragraph::new(" q: Exit | Tab: Switch Panel ")
        .style(Style::default().fg(Color::Gray).bg(Color::Rgb(40, 40, 40)));
    f.render_widget(help_menu, chunks[2]);
}