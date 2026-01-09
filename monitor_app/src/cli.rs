use clap::{Parser, Subcommand};

#[derive(Parser)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    AddServer { name: String, max_ram: f32, port: i32, cpu_model: String },
    RunServer { id: i32 },
    ListServer,
    RemoveServer { id: i32 },
    History,
    Gui,
}