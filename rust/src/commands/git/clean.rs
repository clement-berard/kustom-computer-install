use std::collections::HashMap;
use std::process::Command;
use std::time::Duration;

use console::{style, Term};
use dialoguer::{theme::ColorfulTheme, MultiSelect};
use indicatif::{ProgressBar, ProgressStyle};
use serde_json::Value;

/// How safe it is to delete a branch.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
enum Level {
    /// Nothing is lost: already in the base branch, or its PR is merged.
    Safe,
    /// Probably done, but check: PR closed without merge.
    Review,
    /// Commits would be lost: not merged and no PR proves it is done.
    Risky,
}

struct Branch {
    name: String,
    upstream_gone: bool,
    has_upstream: bool,
    date: String,
    tip: String,
    worktree: String,
}

struct Pr {
    number: u64,
    state: String,
    head_oid: String,
}

struct Candidate {
    name: String,
    level: Level,
    reason: String,
    date: String,
}

enum Verdict {
    Skip(String),
    Pick(Level, String),
}

fn git(args: &[&str]) -> Option<String> {
    let output = Command::new("git").args(args).output().ok()?;
    output
        .status
        .success()
        .then(|| String::from_utf8_lossy(&output.stdout).trim().to_string())
}

/// Like `git`, but keeps trailing tabs (empty last field of a for-each-ref line).
fn git_raw(args: &[&str]) -> Option<String> {
    let output = Command::new("git").args(args).output().ok()?;
    output
        .status
        .success()
        .then(|| String::from_utf8_lossy(&output.stdout).to_string())
}

fn git_ok(args: &[&str]) -> bool {
    Command::new("git")
        .args(args)
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
}

fn spinner(message: &str) -> ProgressBar {
    let bar = ProgressBar::new_spinner();
    bar.set_style(
        ProgressStyle::default_spinner()
            .template("{spinner:.cyan} {msg}")
            .unwrap(),
    );
    bar.set_message(message.to_string());
    bar.enable_steady_tick(Duration::from_millis(80));
    bar
}

/// Remote tracking ref of the default branch, e.g. `origin/main`.
fn default_base() -> Option<String> {
    let remotes = git(&["remote"])?;
    let remote = if remotes.lines().any(|r| r == "origin") {
        "origin"
    } else {
        remotes.lines().next()?
    };
    if let Some(head) = git(&["symbolic-ref", "--short", &format!("refs/remotes/{remote}/HEAD")]) {
        return Some(head);
    }
    ["main", "master"]
        .iter()
        .map(|b| format!("{remote}/{b}"))
        .find(|r| git_ok(&["rev-parse", "--verify", "--quiet", r]))
}

fn local_branches() -> Vec<Branch> {
    let format = "%(refname:short)%09%(upstream)%09%(upstream:track)%09%(committerdate:short)%09%(objectname)%09%(worktreepath)";
    let Some(out) = git_raw(&["for-each-ref", &format!("--format={format}"), "refs/heads/"]) else {
        return Vec::new();
    };
    out.lines()
        .filter_map(|line| {
            let f: Vec<&str> = line.split('\t').collect();
            (f.len() == 6).then(|| Branch {
                name: f[0].to_string(),
                has_upstream: !f[1].is_empty(),
                upstream_gone: f[2].contains("gone"),
                date: f[3].to_string(),
                tip: f[4].to_string(),
                worktree: f[5].to_string(),
            })
        })
        .collect()
}

/// One call for every PR of the repo. The best PR per branch: open, then merged, then closed.
fn pull_requests() -> Result<HashMap<String, Pr>, String> {
    let output = Command::new("gh")
        .args([
            "pr", "list", "--state", "all", "--limit", "1000",
            "--json", "headRefName,state,number,headRefOid",
        ])
        .output()
        .map_err(|_| "gh is not installed".to_string())?;
    if !output.status.success() {
        return Err(String::from_utf8_lossy(&output.stderr).trim().to_string());
    }
    let json: Value = serde_json::from_slice(&output.stdout).map_err(|e| e.to_string())?;

    let rank = |state: &str| match state {
        "OPEN" => 3,
        "MERGED" => 2,
        _ => 1,
    };
    let mut best: HashMap<String, Pr> = HashMap::new();
    for item in json.as_array().into_iter().flatten() {
        let pr = Pr {
            number: item["number"].as_u64().unwrap_or(0),
            state: item["state"].as_str().unwrap_or("").to_string(),
            head_oid: item["headRefOid"].as_str().unwrap_or("").to_string(),
        };
        let name = item["headRefName"].as_str().unwrap_or("").to_string();
        let better = match best.get(&name) {
            None => true,
            Some(old) => (rank(&pr.state), pr.number) > (rank(&old.state), old.number),
        };
        if better {
            best.insert(name, pr);
        }
    }
    Ok(best)
}

/// True when every commit of the branch already exists in the base (same patch).
fn already_applied(base: &str, branch: &str) -> bool {
    git(&["cherry", base, branch])
        .map(|out| !out.is_empty() && out.lines().all(|l| l.starts_with('-')))
        .unwrap_or(false)
}

fn classify(b: &Branch, base: &str, prs: &HashMap<String, Pr>, protected: &[String]) -> Verdict {
    if protected.contains(&b.name) {
        return Verdict::Skip("protected".into());
    }
    if !b.worktree.is_empty() {
        return Verdict::Skip("checked out (current branch or worktree)".into());
    }

    let pr = prs.get(&b.name);
    if let Some(pr) = pr.filter(|p| p.state == "OPEN") {
        return Verdict::Skip(format!("PR #{} is open", pr.number));
    }

    if git_ok(&["merge-base", "--is-ancestor", &b.name, base]) {
        return Verdict::Pick(Level::Safe, format!("merged into {base}"));
    }

    if let Some(pr) = pr.filter(|p| p.state == "MERGED") {
        // The PR must contain the local tip, otherwise local commits would be lost.
        let tip_in_pr = pr.head_oid == b.tip
            || (git_ok(&["cat-file", "-e", &pr.head_oid])
                && git_ok(&["merge-base", "--is-ancestor", &b.tip, &pr.head_oid]));
        return if tip_in_pr {
            Verdict::Pick(Level::Safe, format!("PR #{} merged", pr.number))
        } else {
            Verdict::Pick(
                Level::Risky,
                format!("PR #{} merged, but the branch has newer commits", pr.number),
            )
        };
    }

    if already_applied(base, &b.name) {
        return Verdict::Pick(Level::Safe, "commits already in base (git cherry)".into());
    }

    if let Some(pr) = pr {
        return Verdict::Pick(Level::Review, format!("PR #{} closed without merge", pr.number));
    }

    if b.upstream_gone {
        return Verdict::Pick(Level::Risky, "remote deleted, not merged, no PR".into());
    }
    if !b.has_upstream {
        return Verdict::Pick(Level::Risky, "local only, not merged, no PR".into());
    }
    Verdict::Skip("on the remote, no PR".into())
}

fn badge(level: Level) -> String {
    match level {
        Level::Safe => style("safe  ").green().to_string(),
        Level::Review => style("review").yellow().to_string(),
        Level::Risky => style("risky ").red().to_string(),
    }
}

pub fn run(dry_run: bool, yes: bool) -> Result<(), String> {
    if git(&["rev-parse", "--show-toplevel"]).is_none() {
        return Err("not in a git repository".into());
    }
    let term = Term::stdout();
    if !term.is_term() && !yes && !dry_run {
        return Err("no terminal: use --dry-run or --yes".into());
    }

    let bar = spinner("Fetching and pruning...");
    let fetched = git_ok(&["fetch", "--prune", "--quiet"]);
    bar.finish_and_clear();
    if !fetched {
        eprintln!("{} fetch failed, remote info may be outdated", style("!").yellow());
    }

    let base = default_base().ok_or("cannot find the default branch (origin/main or origin/master)")?;
    let base_short = base.split_once('/').map(|(_, b)| b.to_string()).unwrap_or_else(|| base.clone());
    let protected: Vec<String> = ["main", "master", "develop", "dev"]
        .iter()
        .map(|s| s.to_string())
        .chain([base_short])
        .collect();

    let bar = spinner("Reading pull requests (gh)...");
    let prs = pull_requests();
    bar.finish_and_clear();
    let prs = match prs {
        Ok(prs) => prs,
        Err(e) => {
            eprintln!(
                "{} no PR info ({e}): squash-merged branches may be missed",
                style("!").yellow()
            );
            HashMap::new()
        }
    };

    let mut candidates: Vec<Candidate> = Vec::new();
    let mut kept: Vec<(String, String)> = Vec::new();
    for b in local_branches() {
        match classify(&b, &base, &prs, &protected) {
            Verdict::Skip(reason) => kept.push((b.name, reason)),
            Verdict::Pick(level, reason) => candidates.push(Candidate {
                name: b.name,
                level,
                reason,
                date: b.date,
            }),
        }
    }
    let skipped = kept.len();
    candidates.sort_by(|a, b| a.level.cmp(&b.level).then(a.date.cmp(&b.date)));

    if candidates.is_empty() {
        println!("{} Nothing to clean ({skipped} kept)", style("✓").green());
        return Ok(());
    }

    let label = |c: &Candidate| {
        format!("{}  {}  {}  {}", badge(c.level), c.name, style(&c.reason).dim(), style(&c.date).dim())
    };

    let selected: Vec<usize> = if dry_run {
        for c in &candidates {
            println!("  {}", label(c));
        }
        for (name, reason) in &kept {
            println!("  {}  {}  {}", style("kept  ").dim(), style(name).dim(), style(reason).dim());
        }
        println!(
            "\n{} {} deletable, {skipped} kept (dry run)",
            style("→").yellow(),
            candidates.len()
        );
        return Ok(());
    } else if yes {
        candidates
            .iter()
            .enumerate()
            .filter(|(_, c)| c.level == Level::Safe)
            .map(|(i, _)| i)
            .collect()
    } else {
        let items: Vec<String> = candidates.iter().map(label).collect();
        let defaults: Vec<bool> = candidates.iter().map(|c| c.level == Level::Safe).collect();
        let picked = MultiSelect::with_theme(&ColorfulTheme::default())
            .with_prompt("Branches to delete (space: toggle, enter: confirm, esc: cancel)")
            .items(&items)
            .defaults(&defaults)
            .interact_opt()
            .map_err(|e| e.to_string())?;
        match picked {
            Some(picked) => picked,
            None => {
                println!("Cancelled.");
                return Ok(());
            }
        }
    };

    let mut deleted = 0;
    for i in selected {
        let c = &candidates[i];
        if git_ok(&["branch", "-D", &c.name]) {
            deleted += 1;
            println!("{} {}", style("✓").green(), c.name);
        } else {
            eprintln!("{} {}: delete failed", style("✗").red(), c.name);
        }
    }
    println!("\n{} Deleted {deleted} branch(es), {skipped} kept", style("✓").green());
    Ok(())
}
