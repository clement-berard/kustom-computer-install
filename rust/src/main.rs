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
    /// Filesystem utilities
    Fs {
        #[command(subcommand)]
        subcommand: FsCommands,
    },
}

#[derive(Subcommand)]
enum FsCommands {
    /// Remove node_modules recursively from current directory
    RmNm {
        /// Dry run — show what would be removed without deleting
        #[arg(long)]
        dry_run: bool,
    },
}

fn main() {
    let cli = Cli::parse();

    match cli.command {
        Commands::ToolVersions => commands::tool_versions::run(),
        Commands::Fs { subcommand } => match subcommand {
            FsCommands::RmNm { dry_run } => commands::filesystem::rm_nm::run(dry_run),
        },
    }
}
