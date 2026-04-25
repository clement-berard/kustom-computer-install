use std::process::Command;
use rayon::prelude::*;
use indicatif::{ProgressBar, ProgressStyle};
use comfy_table::{Table, Cell, Color, Attribute};

struct ToolConfig {
    name: &'static str,
    arg: &'static str,
    use_stderr: bool,
}

pub fn run() {
    let tools = vec![
        ToolConfig { name: "brew",  arg: "--version", use_stderr: false },
        ToolConfig { name: "node",  arg: "--version", use_stderr: false },
        ToolConfig { name: "n",  arg: "--version", use_stderr: false },
        ToolConfig { name: "npm",   arg: "--version", use_stderr: false },
        ToolConfig { name: "pnpm",  arg: "--version", use_stderr: false },
        ToolConfig { name: "yarn",  arg: "--version", use_stderr: false },
        ToolConfig { name: "bun",   arg: "--version", use_stderr: false },
        ToolConfig { name: "go",    arg: "version",   use_stderr: false },
    ];

    let spinner = ProgressBar::new_spinner();
    spinner.set_style(
        ProgressStyle::default_spinner()
            .template("{spinner:.cyan} {msg}")
            .unwrap(),
    );
    spinner.set_message("Checking tools...");
    spinner.enable_steady_tick(std::time::Duration::from_millis(80));

    let mut results: Vec<(&str, String)> = tools
        .par_iter()
        .map(|tool| {
            let result = match Command::new(tool.name).arg(tool.arg).output() {
                Ok(output) => {
                    let raw = if tool.use_stderr {
                        String::from_utf8_lossy(&output.stderr).to_string()
                    } else {
                        String::from_utf8_lossy(&output.stdout).to_string()
                    };
                    raw.trim().to_string()
                }
                Err(_) => "not found".to_string(),
            };
            (tool.name, result)
        })
        .collect();

    results.sort_by_key(|(name, _)| {
        tools.iter().position(|t| t.name == *name).unwrap_or(0)
    });

    spinner.finish_and_clear();

    let mut table = Table::new();
    table.set_header(vec![
        Cell::new("Tool").add_attribute(Attribute::Bold).fg(Color::Cyan),
        Cell::new("Version").add_attribute(Attribute::Bold).fg(Color::Cyan),
    ]);

    for (name, version) in &results {
        let version_cell = if *version == "not found" {
            Cell::new(version).fg(Color::Red)
        } else {
            Cell::new(version).fg(Color::Green)
        };
        table.add_row(vec![Cell::new(name), version_cell]);
    }

    println!("{table}");
}
