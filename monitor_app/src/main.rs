mod cli;
mod ui;

use clap::Parser;
use cli::{Cli, Commands};
use Monitor_Lib::db::{establish_connection, get_all_servers};
use Monitor_Lib::engine::simulate_server_metrics;
use ratatui::Terminal;
use std::{io, time::Duration};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut conn = establish_connection();
    let args: Vec<String> = std::env::args().collect();
    
    // Ak nie sú parametre, spusti GUI hneď
    if args.len() <= 1 {
        run_ratatui_loop(&mut conn).await?;
    } else {
        let cli = Cli::parse();
        match cli.command {
            Commands::AddServer { name, max_ram, port, cpu_model } => {
                Monitor_Lib::db::add_server(&mut conn, &name, max_ram, port, &cpu_model);
                println!("Server {} pridany.", name);
            }
            Commands::RunServer { id } => {
                Monitor_Lib::db::update_status(&mut conn, id, "ON");
                println!("Server {} zapnuty.", id);
            }
            _ => run_ratatui_loop(&mut conn).await?,
        }
    }
    Ok(())
}

async fn run_ratatui_loop(conn: &mut Monitor_Lib::db::SqliteConnection) -> Result<(), Box<dyn std::error::Error>> {
    // Inicializácia Ratatui terminálu
    let mut terminal = Terminal::new(ratatui::backend::CrosstermBackend::new(io::stdout()))?;
    
    // Vyčistíme obrazovku pre GUI
    terminal.clear()?;

    loop {
        let servers = get_all_servers(conn).unwrap();
        let mut display_data = Vec::new();

        // Simulácia hodnôt pre každý server v zozname
        for s in servers {
            let (ms, cpu, ram) = simulate_server_metrics(&s).await;
            display_data.push((s, ms, cpu, ram));
        }

        terminal.draw(|f| {
            let table = ui::draw_server_table(&display_data);
            f.render_widget(table, f.area());
        })?;

        // Kontrola vstupu cez standardny event loop Ratatui backendu
        if ratatui::crossterm::event::poll(Duration::from_millis(500))? {
            if let ratatui::crossterm::event::Event::Key(key) = ratatui::crossterm::event::read()? {
                if key.code == ratatui::crossterm::event::KeyCode::Char('q') {
                    break;
                }
            }
        }
    }

    Ok(())
}