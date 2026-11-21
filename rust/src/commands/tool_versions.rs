use std::process::Command;
use std::time::Duration;

use comfy_table::{Attribute, Cell, Color, Table};
use indicatif::{ProgressBar, ProgressStyle};
use rayon::prelude::*;

/// Where the latest version of a tool is looked up.
enum Latest {
    /// npm registry package (its `latest` dist-tag).
    Npm(&'static str),
    /// Latest GitHub release tag of `owner/repo`.
    Github(&'static str),
    /// Latest Node.js release of a major line.
    NodeMajor(u32),
    /// Latest Go release.
    Go,
}

struct Tool {
    category: &'static str,
    label: &'static str,
    cmd: &'static str,
    arg: &'static str,
    latest: Latest,
    /// Brew formula, used when the tool lives in a Homebrew prefix.
    brew: Option<&'static str>,
    /// Update command when the tool is not managed by Homebrew.
    update: &'static str,
}

struct Row {
    category: &'static str,
    label: &'static str,
    installed: Option<String>,
    latest: Option<String>,
    path: Option<String>,
    via_corepack: bool,
    /// Node only: the .node-version / .nvmrc that fnm applies here, if any.
    local: Option<String>,
    update: String,
}

/// Location of the executable (`which`).
fn locate(cmd: &str) -> Option<String> {
    let output = Command::new("which").arg(cmd).output().ok()?;
    if !output.status.success() {
        return None;
    }
    Some(String::from_utf8_lossy(&output.stdout).trim().to_string())
}

/// The `.node-version` / `.nvmrc` found in the current folder or a parent.
fn local_version_file() -> Option<String> {
    let cwd = std::env::current_dir().ok()?;
    cwd.ancestors().find_map(|dir| {
        [".node-version", ".nvmrc"]
            .iter()
            .map(|name| dir.join(name))
            .find(|file| file.is_file())
            .map(|file| short(&file.to_string_lossy()))
    })
}

/// Home folder shortened to `~`.
fn short(path: &str) -> String {
    let home = std::env::var("HOME").unwrap_or_default();
    match path.strip_prefix(&home) {
        Some(rest) if !home.is_empty() => format!("~{rest}"),
        _ => path.to_string(),
    }
}

/// True when the executable is a corepack shim (it resolves into the corepack package).
fn is_corepack_shim(path: &Option<String>) -> bool {
    path.as_deref()
        .and_then(|p| std::fs::canonicalize(p).ok())
        .is_some_and(|real| real.to_string_lossy().contains("/corepack/"))
}

fn update_command(tool: &Tool, path: &Option<String>, via_corepack: bool) -> String {
    if via_corepack {
        return format!("corepack install -g {}@latest", tool.label);
    }
    let from_brew = path
        .as_deref()
        .is_some_and(|p| p.starts_with("/opt/homebrew") || p.starts_with("/usr/local/Cellar"));
    match (from_brew, tool.brew) {
        (true, Some(formula)) => format!("brew upgrade {formula}"),
        _ => tool.update.to_string(),
    }
}

fn fetch(url: &str) -> Option<String> {
    let output = Command::new("curl")
        .args(["-fsSL", "--max-time", "8", url])
        .output()
        .ok()?;
    output
        .status
        .success()
        .then(|| String::from_utf8_lossy(&output.stdout).to_string())
}

/// Reads the string value of `"key"` from a JSON text (first occurrence).
fn json_str(text: &str, key: &str) -> Option<String> {
    let rest = &text[text.find(&format!("\"{key}\""))? + key.len() + 2..];
    let rest = rest.trim_start().strip_prefix(':')?.trim_start().strip_prefix('"')?;
    Some(rest[..rest.find('"')?].to_string())
}

/// First `x.y` or `x.y.z` found in a text ("go version go1.25.1" -> "1.25.1").
fn extract_version(text: &str) -> Option<String> {
    let chars: Vec<char> = text.chars().collect();
    for start in 0..chars.len() {
        if !chars[start].is_ascii_digit() {
            continue;
        }
        let run: String = chars[start..]
            .iter()
            .take_while(|c| c.is_ascii_digit() || **c == '.')
            .collect();
        let run = run.trim_end_matches('.');
        if run.contains('.') {
            return Some(run.to_string());
        }
    }
    None
}

fn installed_version(tool: &Tool) -> Option<String> {
    let output = Command::new(tool.cmd).arg(tool.arg).output().ok()?;
    extract_version(&String::from_utf8_lossy(&output.stdout))
}

fn latest_version(tool: &Tool, installed: &Option<String>, local: &Option<String>) -> Option<String> {
    match &tool.latest {
        Latest::Npm(package) => {
            // Yarn 2+ is published under another package than classic yarn 1.x.
            let package = match (*package, installed) {
                ("yarn", Some(v)) if !v.starts_with("1.") => "@yarnpkg/cli-dist",
                _ => package,
            };
            let body = fetch(&format!("https://registry.npmjs.org/-/package/{package}/dist-tags"))?;
            json_str(&body, "latest")
        }
        Latest::Github(repo) => {
            let body = fetch(&format!("https://api.github.com/repos/{repo}/releases/latest"))?;
            extract_version(&json_str(&body, "tag_name")?)
        }
        Latest::NodeMajor(default_major) => {
            // A local version file may pin another major: look up that one.
            let major = match (local, installed) {
                (Some(_), Some(v)) => parse(v).first().copied().unwrap_or(*default_major as u64),
                _ => *default_major as u64,
            };
            let body = fetch("https://nodejs.org/dist/index.json")?;
            let release = body
                .split("{\"version\"")
                .find(|chunk| chunk.starts_with(&format!(":\"v{major}.")))?;
            extract_version(release)
        }
        Latest::Go => extract_version(&fetch("https://go.dev/VERSION?m=text")?),
    }
}

fn parse(version: &str) -> Vec<u64> {
    version.split('.').filter_map(|p| p.parse().ok()).collect()
}

fn status_cell(row: &Row) -> Cell {
    match (&row.installed, &row.latest) {
        (None, _) => Cell::new("not installed").fg(Color::Red),
        (Some(_), None) => Cell::new("?").fg(Color::DarkGrey),
        (Some(have), Some(last)) => {
            if parse(last) > parse(have) {
                Cell::new("update available").fg(Color::Yellow)
            } else {
                Cell::new("up to date").fg(Color::Green)
            }
        }
    }
}

pub fn run() {
    let term = console::Term::stdout();
    if term.is_term() {
        let _ = term.clear_screen();
    }

    let tools = vec![
        Tool { category: "System", label: "brew", cmd: "brew", arg: "--version", latest: Latest::Github("Homebrew/brew"), brew: None, update: "brew update" },
        Tool { category: "Runtimes", label: "node", cmd: "node", arg: "--version", latest: Latest::NodeMajor(24), brew: Some("node"), update: "fnm install 24" },
        Tool { category: "Runtimes", label: "bun", cmd: "bun", arg: "--version", latest: Latest::Npm("bun"), brew: Some("bun"), update: "bun upgrade" },
        Tool { category: "Runtimes", label: "go", cmd: "go", arg: "version", latest: Latest::Go, brew: Some("go"), update: "install from go.dev/dl" },
        Tool { category: "Runtimes", label: "rust", cmd: "rustc", arg: "--version", latest: Latest::Github("rust-lang/rust"), brew: None, update: "rustup update stable" },
        Tool { category: "Version managers", label: "fnm", cmd: "fnm", arg: "--version", latest: Latest::Github("Schniz/fnm"), brew: Some("fnm"), update: "brew upgrade fnm" },
        Tool { category: "Package managers", label: "corepack", cmd: "corepack", arg: "--version", latest: Latest::Npm("corepack"), brew: None, update: "npm i -g corepack" },
        Tool { category: "Package managers", label: "npm", cmd: "npm", arg: "--version", latest: Latest::Npm("npm"), brew: None, update: "npm i -g npm@latest" },
        Tool { category: "Package managers", label: "pnpm", cmd: "pnpm", arg: "--version", latest: Latest::Npm("pnpm"), brew: Some("pnpm"), update: "pnpm self-update" },
        Tool { category: "Package managers", label: "yarn", cmd: "yarn", arg: "--version", latest: Latest::Npm("yarn"), brew: Some("yarn"), update: "npm i -g yarn" },
    ];

    let spinner = ProgressBar::new_spinner();
    spinner.set_style(
        ProgressStyle::default_spinner()
            .template("{spinner:.cyan} {msg}")
            .unwrap(),
    );
    spinner.set_message("Checking installed and latest versions...");
    spinner.enable_steady_tick(Duration::from_millis(80));

    // Order is kept by `collect` on an indexed parallel iterator.
    let rows: Vec<Row> = tools
        .par_iter()
        .map(|tool| {
            let installed = installed_version(tool);
            let local = if tool.label == "node" { local_version_file() } else { None };
            let latest = latest_version(tool, &installed, &local);
            let path = locate(tool.cmd);
            let via_corepack = tool.label != "corepack" && is_corepack_shim(&path);
            let mut update = update_command(tool, &path, via_corepack);
            if let (Some(_), Some(v)) = (&local, &installed) {
                update = format!("fnm install {}", v.split('.').next().unwrap_or(""));
            }
            Row { category: tool.category, label: tool.label, installed, latest, path, via_corepack, local, update }
        })
        .collect();

    spinner.finish_and_clear();

    let header = |text: &str| Cell::new(text).add_attribute(Attribute::Bold).fg(Color::Cyan);

    // One table per category, in order of first appearance.
    let mut categories: Vec<&str> = Vec::new();
    for row in &rows {
        if !categories.contains(&row.category) {
            categories.push(row.category);
        }
    }

    for category in categories {
        let mut table = Table::new();
        table.set_header(vec![
            header("Tool"),
            header("Installed"),
            header("Latest"),
            header("Status"),
            header("Path"),
            header("Update with"),
        ]);

        for row in rows.iter().filter(|r| r.category == category) {
            let installed = match (&row.installed, &row.local) {
                (Some(v), Some(_)) => format!("{v} (local)"),
                (Some(v), None) => v.clone(),
                (None, _) => "-".to_string(),
            };
            let latest = match (&row.latest, row.label) {
                (Some(v), "node") => format!("{v} ({}.x)", v.split('.').next().unwrap_or("")),
                (Some(v), _) => v.clone(),
                (None, _) => "?".to_string(),
            };
            let outdated = matches!((&row.installed, &row.latest),
                (Some(have), Some(last)) if parse(last) > parse(have));
            // Always shown; dimmed when there is nothing to update.
            let update = if outdated {
                Cell::new(&row.update).fg(Color::Yellow)
            } else {
                Cell::new(&row.update).fg(Color::DarkGrey)
            };
            let mut path = match &row.path {
                Some(p) if row.via_corepack => format!("{} (corepack)", short(p)),
                Some(p) => short(p),
                None => "-".to_string(),
            };
            if let Some(file) = &row.local {
                path.push_str(&format!("\nlocal: {file}"));
            }
            table.add_row(vec![
                Cell::new(row.label),
                Cell::new(installed),
                Cell::new(latest),
                status_cell(row),
                Cell::new(path),
                update,
            ]);
        }

        println!("{}", console::style(category).bold());
        println!("{table}\n");
    }
}
