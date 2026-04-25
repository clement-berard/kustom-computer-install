use std::fs;
use std::path::PathBuf;
use walkdir::WalkDir;

pub fn run(dry_run: bool) {
    let cwd = std::env::current_dir().unwrap();
    let mut found: Vec<PathBuf> = Vec::new();

    println!("📁 Scanning: {}\n", cwd.display());

    let mut it = WalkDir::new(&cwd).into_iter();
    loop {
        match it.next() {
            None => break,
            Some(Err(_)) => continue,
            Some(Ok(entry)) => {
                if entry.file_type().is_dir() && entry.file_name() == "node_modules" {
                    found.push(entry.into_path());
                    it.skip_current_dir();
                }
            }
        }
    }

    if found.is_empty() {
        println!("✓ No node_modules found in {}", cwd.display());
        return;
    }

    for path in &found {
        if dry_run {
            println!("[dry-run] Would remove: {}", path.display());
        } else {
            match fs::remove_dir_all(path) {
                Ok(_) => println!("✓ Removed: {}", path.display()),
                Err(e) => eprintln!("✗ Failed to remove {}: {}", path.display(), e),
            }
        }
    }

    let count = found.len();
    let label = if count == 1 { "directory" } else { "directories" };

    if dry_run {
        println!("\n[dry-run] Would remove {} node_modules {}", count, label);
    } else {
        println!("\n✓ Removed {} node_modules {}", count, label);
    }
}
