use clap::{Parser, Subcommand};

mod commands;

#[derive(Parser)]
#[command(name = "kc")]
#[command(about = "A developer CLI toolbox")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Show versions of common Node.js package managers
    ToolVersions,
    /// Git utilities
    Git {
        #[command(subcommand)]
        subcommand: GitCommands,
    },
    /// Filesystem utilities
    Fs {
        #[command(subcommand)]
        subcommand: FsCommands,
    },
}

#[derive(Subcommand)]
enum GitCommands {
    /// Delete local branches that are merged, gone or abandoned (pick them from a list)
    Clean {
        /// Dry run: list the branches without deleting
        #[arg(long)]
        dry_run: bool,
        /// No prompt: delete only the safe ones
        #[arg(short, long)]
        yes: bool,
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
        Commands::Git { subcommand } => match subcommand {
            GitCommands::Clean { dry_run, yes } => {
                if let Err(e) = commands::git::clean::run(dry_run, yes) {
                    eprintln!("error: {e}");
                    std::process::exit(1);
                }
            }
        },
        Commands::Fs { subcommand } => match subcommand {
            FsCommands::RmNm { dry_run } => commands::filesystem::rm_nm::run(dry_run),
        },
    }
}
