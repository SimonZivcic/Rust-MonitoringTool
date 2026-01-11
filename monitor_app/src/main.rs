mod cli;
mod ui;

use clap::Parser;
use cli::{Cli, Commands};
use Monitor_Lib::db::{establish_connection, get_all_servers, update_status, add_server, remove_server};
use Monitor_Lib::engine::simulate_server_metrics;
use Monitor_Lib::models::Server;
use diesel::prelude::*;
use ratatui::{backend::CrosstermBackend, widgets::TableState, Terminal};
use ratatui::crossterm::{
    event::{self, Event, KeyCode, KeyEventKind},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use std::{collections::HashMap, io::{self, Write}, time::{Duration, Instant}};
use chrono::{DateTime, Utc};
use rand::Rng;
use tokio::sync::mpsc;

#[derive(PartialEq)]
pub enum ActiveBlock {
    Servers,
    Info,
}

#[derive(PartialEq, Clone, Debug)]
pub enum InfoMode {
    View,
    AddServerName,
    AddServerPort,
    AddServerRam,
    AddServerCpu,
    UpdateServerName,
    UpdateServerPort,
    UpdateServerRam,
    UpdateServerCpu,
    ConfirmWarning,
    DeleteConfirm,
}

pub struct AppState {
    pub logs: Vec<String>,
    pub start_times: HashMap<i32, DateTime<Utc>>,
    pub active_block: ActiveBlock,
    pub info_mode: InfoMode,
    pub new_name: String,
    pub new_port: String,
    pub new_ram: String,
    pub new_cpu: String,
    pub update_id: Option<i32>,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut conn = establish_connection();
    diesel::sql_query("PRAGMA journal_mode = WAL;").execute(&mut conn).ok();

    // SPRACOVANIE ARGUMENTOV
    let cli = Cli::parse();

    match cli.command {
        Commands::Gui => {
            run_ratatui_loop(&mut conn).await?;
        }
        Commands::AddServer => {
            println!("--- PRIDANIE SERVERA ---");
            print!("Názov: "); io::stdout().flush()?;
            let mut name = String::new(); io::stdin().read_line(&mut name)?;
            print!("RAM (GB): "); io::stdout().flush()?;
            let mut ram_s = String::new(); io::stdin().read_line(&mut ram_s)?;
            print!("Port: "); io::stdout().flush()?;
            let mut port_s = String::new(); io::stdin().read_line(&mut port_s)?;
            print!("CPU Model: "); io::stdout().flush()?;
            let mut cpu = String::new(); io::stdin().read_line(&mut cpu)?;

            let ram: f32 = ram_s.trim().parse().unwrap_or(0.0);
            let port: i32 = port_s.trim().parse().unwrap_or(0);
            
            add_server(&mut conn, name.trim(), ram, port, cpu.trim());
            println!("Server '{}' bol pridaný.", name.trim());
        }
        Commands::RunServer => {
            print!("Zadaj NÁZOV servera na zapnutie: ");
            io::stdout().flush()?;
            let mut target_name = String::new();
            io::stdin().read_line(&mut target_name)?;
            let target_name = target_name.trim();

            use Monitor_Lib::schema::servers::dsl::*;
            let found_server = servers.filter(name.eq(target_name)).first::<Server>(&mut conn).optional()?;

            if let Some(s) = found_server {
                update_status(&mut conn, s.id, "ON");
                println!("Server '{}' (ID: {}) bol zapnutý.", target_name, s.id);
            } else {
                println!("Chyba: Server s názvom '{}' neexistuje.", target_name);
            }
        }
        Commands::RemoveServer => {
            print!("Zadaj NÁZOV servera na odstránenie: ");
            io::stdout().flush()?;
            let mut target_name = String::new();
            io::stdin().read_line(&mut target_name)?;
            let target_name = target_name.trim();

            use Monitor_Lib::schema::servers::dsl::*;
            let found_server = servers.filter(name.eq(target_name)).first::<Server>(&mut conn).optional()?;

            if let Some(s) = found_server {
                remove_server(&mut conn, s.id).ok();
                println!("Server '{}' (ID: {}) bol odstránený.", target_name, s.id);
            } else {
                println!("Chyba: Server s názvom '{}' neexistuje.", target_name);
            }
        }
        Commands::ListServer => {
            let servers_list = get_all_servers(&mut conn).unwrap();
            println!("{:-<60}", "");
            println!("{:<5} | {:<10} | {:<20} | {:<5}", "ID", "STAV", "NÁZOV", "PORT");
            println!("{:-<60}", "");
            for s in servers_list {
                println!("{:<5} | {:<10} | {:<20} | {:<5}", s.id, s.status, s.name, s.port);
            }
        }
        Commands::UpdateServer => {
            print!("Zadaj NÁZOV servera na úpravu: ");
            io::stdout().flush()?;
            let mut target_name = String::new();
            io::stdin().read_line(&mut target_name)?;
            let target_name = target_name.trim();

            use Monitor_Lib::schema::servers::dsl::*;
            let found_server = servers.filter(name.eq(target_name)).first::<Server>(&mut conn).optional()?;

            if let Some(s) = found_server {
                println!("--- ÚPRAVA SERVERA (ID: {}, Aktuálne meno: {}) ---", s.id, s.name);
                println!("(Pre zachovanie pôvodnej hodnoty stlačte ENTER)");

                // 1. NOVÝ NÁZOV
                print!("Nový názov [{}]: ", s.name);
                io::stdout().flush()?;
                let mut n_name = String::new();
                io::stdin().read_line(&mut n_name)?;
                let final_name = if n_name.trim().is_empty() { s.name } else { n_name.trim().to_string() };

                // 2. NOVÝ PORT
                print!("Nový port [{}]: ", s.port);
                io::stdout().flush()?;
                let mut n_port = String::new();
                io::stdin().read_line(&mut n_port)?;
                let final_port = n_port.trim().parse::<i32>().unwrap_or(s.port);

                // 3. NOVÁ RAM
                print!("Nová RAM v GB [{:.1}]: ", s.max_ram);
                io::stdout().flush()?;
                let mut n_ram = String::new();
                io::stdin().read_line(&mut n_ram)?;
                let final_ram = n_ram.trim().parse::<f32>().unwrap_or(s.max_ram);

                // 4. NOVÉ CPU
                print!("Nový CPU model [{}]: ", s.cpu_model);
                io::stdout().flush()?;
                let mut n_cpu = String::new();
                io::stdin().read_line(&mut n_cpu)?;
                let final_cpu = if n_cpu.trim().is_empty() { s.cpu_model } else { n_cpu.trim().to_string() };

                // ZÁPIS DO DB
                diesel::update(servers.filter(id.eq(s.id)))
                    .set((
                        name.eq(final_name),
                        port.eq(final_port),
                        max_ram.eq(final_ram),
                        cpu_model.eq(final_cpu)
                    ))
                    .execute(&mut conn)?;

                println!("Server bol úspešne aktualizovaný.");
            } else {
                println!("Chyba: Server s názvom '{}' neexistuje.", target_name);
            }
        }
    }

    Ok(())
}

async fn run_ratatui_loop(conn: &mut Monitor_Lib::db::SqliteConnection) -> Result<(), Box<dyn std::error::Error>> {
    enable_raw_mode()?;
    execute!(io::stdout(), EnterAlternateScreen)?;
    let mut terminal = Terminal::new(CrosstermBackend::new(io::stdout()))?;

    let mut state = TableState::default();
    state.select(Some(0));

    let mut app_state = AppState {
        logs: vec![format!("[{}] Monitoring beží", Utc::now().format("%H:%M:%S"))],
        start_times: HashMap::new(),
        active_block: ActiveBlock::Servers,
        info_mode: InfoMode::View,
        new_name: String::new(),
        new_port: String::new(),
        new_ram: String::new(),
        new_cpu: String::new(),
        update_id: None,
    };

    //PREPNUTIE STAVOV PRI ŠTARTE
    let startup_servers = get_all_servers(conn).unwrap();
    for s in startup_servers {
        if s.status == "STARTING" {
            update_status(conn, s.id, "ON");
            app_state.logs.push(format!("[{}] {}: Stav opravený na ON", Utc::now().format("%H:%M:%S"), s.name));
            app_state.start_times.insert(s.id, Utc::now());
        } else if s.status == "STOPPING" {
            update_status(conn, s.id, "OFF");
            app_state.logs.push(format!("[{}] {}: Stav opravený na OFF", Utc::now().format("%H:%M:%S"), s.name));
        } else if s.status == "ON" { 
            app_state.start_times.insert(s.id, Utc::now()); 
        }
    }

    let (tx, mut rx) = mpsc::channel::<(i32, String, bool)>(100);
    let valid_ports = vec![80, 443, 3000, 8080, 27017];
    let valid_cpus = vec!["intel-i5", "intel-i7", "intel-i9", "ryzen-5", "ryzen-7", "ryzen-9"];

    loop {
        while let Ok((id, log_msg, is_on)) = rx.try_recv() {
            app_state.logs.push(log_msg);
            if is_on { app_state.start_times.insert(id, Utc::now()); }
            else { app_state.start_times.remove(&id); }
        }

        let servers_list = get_all_servers(conn).unwrap();
        let mut display_data = Vec::new();
        for s in servers_list {
            let (ms, cpu, ram) = simulate_server_metrics(&s).await;
            display_data.push((s, ms, cpu, ram));
        }

        terminal.draw(|f| ui::draw_main_layout(f, &display_data, &mut state, &app_state))?;

        if event::poll(Duration::from_millis(100))? {
            if let Event::Key(key) = event::read()? {
                if key.kind == KeyEventKind::Press {
                    if app_state.active_block == ActiveBlock::Info && !matches!(app_state.info_mode, InfoMode::View | InfoMode::ConfirmWarning | InfoMode::DeleteConfirm) {
                        match key.code {
                            KeyCode::Char(c) => {
                                match app_state.info_mode {
                                    InfoMode::AddServerName | InfoMode::UpdateServerName => app_state.new_name.push(c),
                                    InfoMode::AddServerPort | InfoMode::UpdateServerPort => if c.is_digit(10) { app_state.new_port.push(c) },
                                    InfoMode::AddServerRam | InfoMode::UpdateServerRam => if c.is_digit(10) || c == '.' { app_state.new_ram.push(c) },
                                    InfoMode::AddServerCpu | InfoMode::UpdateServerCpu => app_state.new_cpu.push(c),
                                    _ => {}
                                }
                                continue;
                            }
                            KeyCode::Backspace => {
                                match app_state.info_mode {
                                    InfoMode::AddServerName | InfoMode::UpdateServerName => { app_state.new_name.pop(); }
                                    InfoMode::AddServerPort | InfoMode::UpdateServerPort => { app_state.new_port.pop(); }
                                    InfoMode::AddServerRam | InfoMode::UpdateServerRam => { app_state.new_ram.pop(); }
                                    InfoMode::AddServerCpu | InfoMode::UpdateServerCpu => { app_state.new_cpu.pop(); }
                                    _ => {}
                                }
                                continue;
                            }
                            _ => {}
                        }
                    }

                    match key.code {
                        KeyCode::Char('q') => break,
                        KeyCode::Char('a') => {
                            if app_state.active_block == ActiveBlock::Servers {
                                if let Some(idx) = state.selected() {
                                    if let Some((s, _, _, _)) = display_data.get(idx) {
                                        if s.status == "/" {
                                            let sid = s.id;
                                            let sname = s.name.clone();
                                            let tx_clone = tx.clone();
                                            tokio::spawn(async move {
                                                for i in (0..=100).step_by(25) {
                                                    let _ = tx_clone.send((sid, format!("[{}] {}: Aktivácia {}%", Utc::now().format("%H:%M:%S"), sname, i), false)).await;
                                                    tokio::time::sleep(Duration::from_millis(400)).await;
                                                }
                                                let mut bg_conn = establish_connection();
                                                update_status(&mut bg_conn, sid, "OFF");
                                                let _ = tx_clone.send((sid, format!("[{}] {}: Activation complete", Utc::now().format("%H:%M:%S"), sname), false)).await;
                                            });
                                        } else {
                                            app_state.logs.push(format!("[{}] ERROR: Server {} je už aktivovaný!", Utc::now().format("%H:%M:%S"), s.name));
                                        }
                                    }
                                }
                            }
                        }
                        KeyCode::Char('r') => {
                            if app_state.active_block == ActiveBlock::Servers {
                                if let Some(idx) = state.selected() {
                                    if let Some((s, _, _, _)) = display_data.get(idx) {
                                        app_state.update_id = Some(s.id);
                                        app_state.new_name = s.name.clone();
                                        app_state.info_mode = InfoMode::DeleteConfirm;
                                        app_state.active_block = ActiveBlock::Info;
                                    }
                                }
                            }
                        }
                        KeyCode::Char('n') => {
                            if app_state.active_block == ActiveBlock::Info {
                                app_state.info_mode = InfoMode::AddServerName;
                                app_state.update_id = None;
                                app_state.new_name.clear(); app_state.new_port.clear();
                                app_state.new_ram.clear(); app_state.new_cpu.clear();
                            }
                        }
                        KeyCode::Char('u') => {
                            if app_state.active_block == ActiveBlock::Info {
                                if let Some(idx) = state.selected() {
                                    if let Some((s, _, _, _)) = display_data.get(idx) {
                                        if s.status == "OFF" {
                                            app_state.info_mode = InfoMode::UpdateServerName;
                                            app_state.update_id = Some(s.id);
                                            app_state.new_name = s.name.clone();
                                            app_state.new_port = s.port.to_string();
                                            app_state.new_ram = s.max_ram.to_string();
                                            app_state.new_cpu = s.cpu_model.clone();
                                        } else {
                                            app_state.logs.push(format!("[{}] ERROR: Server musí byť OFF pre úpravu!", Utc::now().format("%H:%M:%S")));
                                        }
                                    }
                                }
                            }
                        }
                        KeyCode::Tab => {
                            app_state.active_block = match app_state.active_block {
                                ActiveBlock::Servers => ActiveBlock::Info,
                                ActiveBlock::Info => { app_state.info_mode = InfoMode::View; ActiveBlock::Servers },
                            };
                        }
                        KeyCode::Down | KeyCode::Up => {
                            let len = display_data.len();
                            if app_state.active_block == ActiveBlock::Info && !matches!(app_state.info_mode, InfoMode::View | InfoMode::DeleteConfirm | InfoMode::ConfirmWarning) {
                                if key.code == KeyCode::Down {
                                    app_state.info_mode = match app_state.info_mode {
                                        InfoMode::AddServerName | InfoMode::UpdateServerName => InfoMode::AddServerPort,
                                        InfoMode::AddServerPort | InfoMode::UpdateServerPort => InfoMode::AddServerRam,
                                        InfoMode::AddServerRam | InfoMode::UpdateServerRam => InfoMode::AddServerCpu,
                                        _ => app_state.info_mode.clone(),
                                    };
                                } else {
                                    app_state.info_mode = match app_state.info_mode {
                                        InfoMode::AddServerPort | InfoMode::UpdateServerPort => InfoMode::AddServerName,
                                        InfoMode::AddServerRam | InfoMode::UpdateServerRam => InfoMode::AddServerPort,
                                        InfoMode::AddServerCpu | InfoMode::UpdateServerCpu => InfoMode::AddServerRam,
                                        _ => app_state.info_mode.clone(),
                                    };
                                }
                            } else if len > 0 {
                                let i = match key.code {
                                    KeyCode::Down => state.selected().map(|i| (i + 1) % len).unwrap_or(0),
                                    _ => state.selected().map(|i| if i == 0 { len - 1 } else { i - 1 }).unwrap_or(0),
                                };
                                state.select(Some(i));
                            }
                        }
                        KeyCode::Enter => {
                            if app_state.active_block == ActiveBlock::Info {
                                match app_state.info_mode {
                                    InfoMode::DeleteConfirm => {
                                        if let Some(uid) = app_state.update_id {
                                            remove_server(conn, uid).ok();
                                            app_state.logs.push(format!("[{}] SERVER ODSTRÁNENÝ: {}", Utc::now().format("%H:%M:%S"), app_state.new_name));
                                            app_state.info_mode = InfoMode::View;
                                            app_state.active_block = ActiveBlock::Servers;
                                        }
                                    }
                                    InfoMode::AddServerName | InfoMode::UpdateServerName => app_state.info_mode = if app_state.update_id.is_some() { InfoMode::UpdateServerPort } else { InfoMode::AddServerPort },
                                    InfoMode::AddServerPort | InfoMode::UpdateServerPort => app_state.info_mode = if app_state.update_id.is_some() { InfoMode::UpdateServerRam } else { InfoMode::AddServerRam },
                                    InfoMode::AddServerRam | InfoMode::UpdateServerRam => app_state.info_mode = if app_state.update_id.is_some() { InfoMode::UpdateServerCpu } else { InfoMode::AddServerCpu },
                                    InfoMode::AddServerCpu | InfoMode::UpdateServerCpu | InfoMode::ConfirmWarning => {
                                        let p: i32 = app_state.new_port.parse().unwrap_or(0);
                                        let r: f32 = app_state.new_ram.parse().unwrap_or(0.0);
                                        let p_ok = valid_ports.contains(&p);
                                        let c_ok = valid_cpus.contains(&app_state.new_cpu.as_str());

                                        if (p_ok && c_ok) || app_state.info_mode == InfoMode::ConfirmWarning {
                                            let detail_msg = format!("{} (Port: {}, RAM: {}G, CPU: {})", app_state.new_name, p, r, app_state.new_cpu);
                                            
                                            if let Some(uid) = app_state.update_id {
                                                use Monitor_Lib::schema::servers::dsl::*;
                                                diesel::update(servers.filter(id.eq(uid))).set((name.eq(&app_state.new_name), port.eq(p), max_ram.eq(r), cpu_model.eq(&app_state.new_cpu))).execute(conn).ok();
                                                app_state.logs.push(format!("[{}] SERVER AKTUALIZOVANÝ: {}", Utc::now().format("%H:%M:%S"), detail_msg));
                                            } else {
                                                add_server(conn, &app_state.new_name, r, p, &app_state.new_cpu);
                                                app_state.logs.push(format!("[{}] SERVER VYTVORENÝ: {}", Utc::now().format("%H:%M:%S"), detail_msg));
                                            }
                                            app_state.info_mode = InfoMode::View;
                                        } else {
                                            app_state.info_mode = InfoMode::ConfirmWarning;
                                        }
                                    }
                                    _ => {}
                                }
                            } else if app_state.active_block == ActiveBlock::Servers {
                                if let Some(idx) = state.selected() {
                                    if let Some((s, _, _, _)) = display_data.get(idx) {
                                        if s.status == "/" {
                                            app_state.logs.push(format!("[{}] ERROR: Server nie je aktivovaný!", Utc::now().format("%H:%M:%S")));
                                        } else if s.status == "ON" || s.status == "OFF" {
                                            let sid = s.id;
                                            let sname = s.name.clone();
                                            let is_turning_on = s.status == "OFF";
                                            
                                            let start_msg = if is_turning_on { "Starting" } else { "Stopping" };
                                            app_state.logs.push(format!("[{}] {}: {}", Utc::now().format("%H:%M:%S"), sname, start_msg));

                                            let (temp, final_s, final_log) = if is_turning_on { ("Starting", "ON", "Started") } else { ("Stopping", "OFF", "Stopped") };
                                            update_status(conn, sid, temp);
                                            
                                            let tx_clone = tx.clone();
                                            tokio::spawn(async move {
                                                let start_inst = Instant::now();
                                                tokio::time::sleep(Duration::from_secs(3)).await;
                                                let duration = start_inst.elapsed().as_secs();
                                                
                                                let mut bg_conn = establish_connection();
                                                update_status(&mut bg_conn, sid, final_s);
                                                
                                                let log_line = format!("[{}] {}: {} (trvanie: {}s)", Utc::now().format("%H:%M:%S"), sname, final_log, duration);
                                                let _ = tx_clone.send((sid, log_line, final_s == "ON")).await;
                                            });
                                        }
                                    }
                                }
                            }
                        }
                        KeyCode::Esc => { 
                            if app_state.info_mode == InfoMode::ConfirmWarning {
                                app_state.info_mode = if app_state.update_id.is_some() { InfoMode::UpdateServerCpu } else { InfoMode::AddServerCpu };
                            } else {
                                app_state.info_mode = InfoMode::View; 
                            }
                        }
                        _ => {}
                    }
                }
            }
        }
    }
    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    Ok(())
}