use clap::{Parser, Subcommand};

mod commands;

#[derive(Parser)]
#[command(name = "devtool")]
#[command(about = "A developer CLI toolbox")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Show versions of common Node.js package managers
    ToolVersions,
}

fn main() {
    let cli = Cli::parse();

    match cli.command {
        Commands::ToolVersions => commands::tool_versions::run(),
    }
}
