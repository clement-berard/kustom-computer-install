use std::fs;
use std::path::{Path, PathBuf};
use std::time::Duration;

use console::style;
use indicatif::{MultiProgress, ProgressBar, ProgressStyle};
use rayon::prelude::*;
use walkdir::WalkDir;

fn spinner(message: String) -> ProgressBar {
    let bar = ProgressBar::new_spinner();
    bar.set_style(
        ProgressStyle::default_spinner()
            .template("{spinner:.cyan} {msg}")
            .unwrap(),
    );
    bar.set_message(message);
    bar.enable_steady_tick(Duration::from_millis(80));
    bar
}

fn dir_size(path: &Path) -> u64 {
    WalkDir::new(path)
        .into_iter()
        .filter_map(Result::ok)
        .filter_map(|entry| entry.metadata().ok())
        .filter(|meta| meta.is_file())
        .map(|meta| meta.len())
        .sum()
}

fn human(bytes: u64) -> String {
    const UNITS: [&str; 4] = ["B", "KB", "MB", "GB"];
    let mut value = bytes as f64;
    let mut unit = 0;
    while value >= 1024.0 && unit < UNITS.len() - 1 {
        value /= 1024.0;
        unit += 1;
    }
    if unit == 0 {
        format!("{bytes} B")
    } else {
        format!("{value:.1} {}", UNITS[unit])
    }
}

fn scan(cwd: &Path) -> Vec<PathBuf> {
    let bar = spinner(format!("Scanning {}", cwd.display()));
    let mut found = Vec::new();

    let mut it = WalkDir::new(cwd).into_iter();
    while let Some(next) = it.next() {
        let Ok(entry) = next else { continue };
        if entry.file_type().is_dir() && entry.file_name() == "node_modules" {
            found.push(entry.into_path());
            it.skip_current_dir();
            bar.set_message(format!(
                "Scanning {} ({} found)",
                cwd.display(),
                found.len()
            ));
        }
    }

    bar.finish_and_clear();
    found
}

fn measure(found: Vec<PathBuf>) -> Vec<(PathBuf, u64)> {
    let bar = spinner("Measuring sizes".to_string());
    let mut sized: Vec<(PathBuf, u64)> = found
        .into_par_iter()
        .map(|path| {
            let size = dir_size(&path);
            (path, size)
        })
        .collect();
    bar.finish_and_clear();

    sized.sort_by(|a, b| b.1.cmp(&a.1));
    sized
}

pub fn run(dry_run: bool) {
    let cwd = std::env::current_dir().unwrap();
    let found = scan(&cwd);

    if found.is_empty() {
        println!("{} No node_modules found", style("✓").green());
        return;
    }

    let sized = measure(found);
    let total: u64 = sized.iter().map(|(_, size)| size).sum();
    let count = sized.len();
    let label = if count == 1 { "folder" } else { "folders" };
    let relative = |path: &Path| path.strip_prefix(&cwd).unwrap_or(path).display().to_string();

    if dry_run {
        for (path, size) in &sized {
            println!(
                "  {} {}  {}",
                style("•").dim(),
                relative(path),
                style(human(*size)).dim()
            );
        }
        println!(
            "\n{} {count} {label}, {} would be freed (dry run)",
            style("→").yellow(),
            style(human(total)).bold()
        );
        return;
    }

    // One spinner per folder: they are removed in parallel.
    let multi = MultiProgress::new();
    let line_style = ProgressStyle::default_spinner()
        .template("{spinner:.cyan} {msg}")
        .unwrap();
    let jobs: Vec<(PathBuf, u64, ProgressBar)> = sized
        .into_iter()
        .map(|(path, size)| {
            let bar = multi.add(ProgressBar::new_spinner());
            bar.set_style(line_style.clone());
            bar.set_message(format!("{}  {}", relative(&path), style(human(size)).dim()));
            bar.enable_steady_tick(Duration::from_millis(80));
            (path, size, bar)
        })
        .collect();

    let results: Vec<Option<u64>> = jobs
        .into_par_iter()
        .map(|(path, size, bar)| {
            let name = relative(&path);
            match fs::remove_dir_all(&path) {
                Ok(_) => {
                    bar.set_style(ProgressStyle::default_spinner().template("{msg}").unwrap());
                    bar.finish_with_message(format!(
                        "{} {name}  {}",
                        style("✓").green(),
                        style(human(size)).dim()
                    ));
                    Some(size)
                }
                Err(e) => {
                    bar.set_style(ProgressStyle::default_spinner().template("{msg}").unwrap());
                    bar.finish_with_message(format!("{} {name}: {e}", style("✗").red()));
                    None
                }
            }
        })
        .collect();

    let removed = results.iter().flatten().count();
    let freed: u64 = results.iter().flatten().sum();
    let removed_label = if removed == 1 { "folder" } else { "folders" };
    println!(
        "\n{} Removed {removed} {removed_label}, {} freed",
        style("✓").green(),
        style(human(freed)).bold()
    );
}
