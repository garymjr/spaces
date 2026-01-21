mod cli;
mod clone;
mod config;
mod copy;
mod git;
mod hooks;
mod mirror;
mod paths;
mod targets;
mod ui;

use std::collections::HashMap;
use std::path::{Path, PathBuf};

use anyhow::{bail, Result};
use clap::Parser;

use cli::{CleanArgs, Commands, CopyArgs, NewArgs, RmArgs, RunArgs};

fn main() {
    if let Err(err) = run() {
        ui::log_error(&err.to_string());
        std::process::exit(1);
    }
}

fn run() -> Result<()> {
    let cli = cli::Cli::parse();
    match cli.command {
        Commands::New(args) => cmd_new(args),
        Commands::Rm(args) => cmd_rm(args),
        Commands::Go(args) => cmd_go(&args.id),
        Commands::Run(args) => cmd_run(args),
        Commands::List(args) => cmd_list(args),
        Commands::Copy(args) => cmd_copy(args),
        Commands::Clean(args) => cmd_clean(args),
        Commands::Doctor => cmd_doctor(),
        Commands::Mirrors(args) => cmd_mirrors(args),
        Commands::Config(args) => cmd_config(args),
    }
}

fn cmd_new(args: NewArgs) -> Result<()> {
    let repo_root = paths::repo_root()?;
    let clones_dir = paths::clones_dir(&repo_root)?;
    let prefix = paths::clones_prefix(&repo_root)?;
    let mirror_dir = paths::mirror_dir(&repo_root)?;

    if args.from.is_some() && args.branch.is_none() {
        bail!("--from requires --branch");
    }

    let mut space = args.name.clone();
    if space.is_none() {
        if args.yes {
            bail!("Space name required in non-interactive mode");
        }
        let input = ui::prompt_input("Enter space name:")?;
        if input.is_empty() {
            bail!("Space name required");
        }
        space = Some(input);
    }
    let space = space.unwrap();
    let folder_name = paths::sanitize_branch_name(&space);
    let clone_path = clones_dir.join(format!("{prefix}{folder_name}"));

    ui::log_step(&format!("Creating space: {space}"));
    eprintln!("Location: {}", clone_path.display());
    eprintln!("Space: {space}");
    if let Some(branch) = &args.branch {
        eprintln!("Branch: {branch}");
    }

    mirror::ensure_mirror(&repo_root, &mirror_dir)?;
    if !args.no_fetch {
        mirror::update_mirror(&repo_root, &mirror_dir)?;
    }

    let base_ref = match &args.from {
        Some(value) => value.clone(),
        None => paths::default_branch(&repo_root)?,
    };
    let plan = clone::ClonePlan {
        path: clone_path.clone(),
        branch: args.branch.clone(),
        base_ref,
    };

    clone::create_clone(&repo_root, &mirror_dir, &plan)?;

    if !args.no_copy {
        let mut includes = config::cfg_get_all("spaces.copy.include", config::Scope::Auto, &repo_root);
        let worktree_include = copy::parse_pattern_file(&repo_root.join(".worktreeinclude"))?;
        let spaces_include = copy::parse_pattern_file(&repo_root.join(".spacesinclude"))?;
        includes.extend(worktree_include);
        includes.extend(spaces_include);
        includes = dedupe(includes);

        let excludes = config::cfg_get_all("spaces.copy.exclude", config::Scope::Auto, &repo_root);

        if !includes.is_empty() {
            ui::log_step("Copying files...");
            copy::copy_patterns(&repo_root, &clone_path, &includes, &excludes, false)?;
        }

        let dir_includes = config::cfg_get_all("spaces.copy.includeDirs", config::Scope::Auto, &repo_root);
        let dir_excludes = config::cfg_get_all("spaces.copy.excludeDirs", config::Scope::Auto, &repo_root);
        if !dir_includes.is_empty() {
            ui::log_step("Copying directories...");
            copy::copy_directories(&repo_root, &clone_path, &dir_includes, &dir_excludes)?;
        }
    }

    let mut envs = HashMap::new();
    envs.insert("REPO_ROOT".to_string(), repo_root.to_string_lossy().to_string());
    envs.insert("CLONE_PATH".to_string(), clone_path.to_string_lossy().to_string());
    envs.insert("SPACE".to_string(), space.clone());
    if let Some(branch) = targets::current_branch(&clone_path) {
        envs.insert("BRANCH".to_string(), branch);
    }

    hooks::run_hooks("postCreate", &repo_root, &clone_path, &envs)?;

    ui::log_info(&format!("Space created: {}", clone_path.display()));
    Ok(())
}

fn cmd_rm(args: RmArgs) -> Result<()> {
    if args.targets.is_empty() {
        bail!("Usage: spaces rm <space|id> [<space|id>...] [--force] [--yes]");
    }

    let repo_root = paths::repo_root()?;
    let clones_dir = paths::clones_dir(&repo_root)?;
    let prefix = paths::clones_prefix(&repo_root)?;

    for identifier in args.targets {
        let target = targets::resolve_target(&identifier, &repo_root, &clones_dir, &prefix)?;
        if target.is_main {
            ui::log_error("Cannot remove main repository");
            continue;
        }

        ui::log_step(&format!("Removing space: {}", target.path.display()));

        let mut envs = HashMap::new();
        envs.insert("REPO_ROOT".to_string(), repo_root.to_string_lossy().to_string());
        envs.insert("CLONE_PATH".to_string(), target.path.to_string_lossy().to_string());
        envs.insert("SPACE".to_string(), target.name.clone());
        envs.insert("BRANCH".to_string(), target.branch.clone());

        if let Err(err) = hooks::run_hooks("preRemove", &repo_root, &target.path, &envs) {
            if !args.force {
                ui::log_error(&format!("Pre-remove hook failed: {err}"));
                continue;
            }
            ui::log_warn("Pre-remove hook failed; continuing due to --force");
        }

        safe_remove_clone(&target.path, &clones_dir)?;

        let _ = hooks::run_hooks("postRemove", &repo_root, &repo_root, &envs);
    }

    Ok(())
}

fn cmd_go(id: &str) -> Result<()> {
    let repo_root = paths::repo_root()?;
    let clones_dir = paths::clones_dir(&repo_root)?;
    let prefix = paths::clones_prefix(&repo_root)?;

    let target = targets::resolve_target(id, &repo_root, &clones_dir, &prefix)?;
    if target.is_main {
        eprintln!("Main repo");
    } else {
        eprintln!("Space: {}", target.name);
    }
    eprintln!("Branch: {}", target.branch);
    println!("{}", target.path.display());
    Ok(())
}

fn cmd_run(args: RunArgs) -> Result<()> {
    if args.cmd.is_empty() {
        bail!("Usage: spaces run <space|id> -- <command...>");
    }

    let repo_root = paths::repo_root()?;
    let clones_dir = paths::clones_dir(&repo_root)?;
    let prefix = paths::clones_prefix(&repo_root)?;
    let target = targets::resolve_target(&args.id, &repo_root, &clones_dir, &prefix)?;

    ui::log_step(&format!("Running in: {}", target.name));
    eprintln!("Command: {}", args.cmd.join(" "));
    eprintln!();

    let mut cmd = std::process::Command::new(&args.cmd[0]);
    if args.cmd.len() > 1 {
        cmd.args(&args.cmd[1..]);
    }
    cmd.current_dir(&target.path);
    let status = cmd.status()?;
    if !status.success() {
        bail!("Command failed")
    }
    Ok(())
}

fn cmd_list(args: cli::ListArgs) -> Result<()> {
    let repo_root = paths::repo_root()?;
    let clones_dir = paths::clones_dir(&repo_root)?;
    let prefix = paths::clones_prefix(&repo_root)?;

    if args.porcelain {
        let branch = targets::current_branch(&repo_root).unwrap_or_else(|| "(detached)".to_string());
        let status = targets::status(&repo_root);
        println!("{}\t{}\t{}\t{}", repo_root.display(), "main", branch, status);

        if clones_dir.is_dir() {
            let mut entries: Vec<PathBuf> = std::fs::read_dir(&clones_dir)?
                .filter_map(|entry| entry.ok().map(|e| e.path()))
                .filter(|p| p.is_dir())
                .filter(|p| p.file_name().and_then(|s| s.to_str()).map(|n| n.starts_with(&prefix)).unwrap_or(false))
                .collect();
            entries.sort();

            for path in entries {
                let branch = targets::current_branch(&path).unwrap_or_else(|| "(detached)".to_string());
                let status = targets::status(&path);
                let name = targets::space_name(&path, &prefix);
                println!("{}\t{}\t{}\t{}", path.display(), name, branch, status);
            }
        }
        return Ok(());
    }

    println!("Spaces");
    println!();
    println!("{:<24} {:<24} {}", "SPACE", "BRANCH", "PATH");
    println!("{:<24} {:<24} {}", "-----", "------", "----");

    let branch = targets::current_branch(&repo_root).unwrap_or_else(|| "(detached)".to_string());
    println!("{:<24} {:<24} {}", "main", branch, repo_root.display());

    if clones_dir.is_dir() {
        let mut rows = Vec::new();
        for entry in std::fs::read_dir(&clones_dir)? {
            let entry = entry?;
            if !entry.file_type()?.is_dir() {
                continue;
            }
            let path = entry.path();
            if let Some(name) = path.file_name().and_then(|s| s.to_str()) {
                if !name.starts_with(&prefix) {
                    continue;
                }
            }
            let branch = targets::current_branch(&path).unwrap_or_else(|| "(detached)".to_string());
            let name = targets::space_name(&path, &prefix);
            rows.push((name, branch, path));
        }
        rows.sort_by(|a, b| a.0.cmp(&b.0));
        for (name, branch, path) in rows {
            println!("{:<24} {:<24} {}", name, branch, path.display());
        }
    }

    println!();
    println!("Tip: Use 'spaces list --porcelain' for machine-readable output");
    Ok(())
}

fn cmd_copy(args: CopyArgs) -> Result<()> {
    let repo_root = paths::repo_root()?;
    let clones_dir = paths::clones_dir(&repo_root)?;
    let prefix = paths::clones_prefix(&repo_root)?;

    let source = args.from.unwrap_or_else(|| "1".to_string());
    let src_target = targets::resolve_target(&source, &repo_root, &clones_dir, &prefix)?;

    let mut patterns = args.patterns.clone();
    if patterns.is_empty() {
        let mut includes = config::cfg_get_all("spaces.copy.include", config::Scope::Auto, &repo_root);
        let worktree_include = copy::parse_pattern_file(&repo_root.join(".worktreeinclude"))?;
        let spaces_include = copy::parse_pattern_file(&repo_root.join(".spacesinclude"))?;
        includes.extend(worktree_include);
        includes.extend(spaces_include);
        patterns = dedupe(includes);
    }

    if patterns.is_empty() {
        bail!("No patterns specified. Use '-- <pattern>...' or configure spaces.copy.include");
    }

    let excludes = config::cfg_get_all("spaces.copy.exclude", config::Scope::Auto, &repo_root);

    let targets = if args.all {
        list_space_names(&clones_dir, &prefix)?
    } else {
        args.targets
    };

    if !args.all && targets.is_empty() {
        bail!("Usage: spaces copy <space>... [-n] [-a] [--from <space>] [-- <pattern>...]");
    }

    let mut copied = false;
    for target_id in targets {
        let dst_target = targets::resolve_target(&target_id, &repo_root, &clones_dir, &prefix)?;
        if dst_target.path == src_target.path {
            continue;
        }
        if args.dry_run {
            ui::log_step(&format!("[dry-run] Would copy to: {}", dst_target.name));
            copy::copy_patterns(&src_target.path, &dst_target.path, &patterns, &excludes, true)?;
        } else {
            ui::log_step(&format!("Copying to: {}", dst_target.name));
            copy::copy_patterns(&src_target.path, &dst_target.path, &patterns, &excludes, false)?;
        }
        copied = true;
    }

    if !copied {
        ui::log_warn("No files copied (source and target may be the same)");
    }

    Ok(())
}

fn cmd_clean(args: CleanArgs) -> Result<()> {
    let repo_root = paths::repo_root()?;
    let clones_dir = paths::clones_dir(&repo_root)?;
    let prefix = paths::clones_prefix(&repo_root)?;

    ui::log_step("Cleaning up stale spaces...");

    if clones_dir.is_dir() {
        for entry in std::fs::read_dir(&clones_dir)? {
            let entry = entry?;
            if entry.path() == clones_dir {
                continue;
            }
            if entry.file_type()?.is_dir() {
                let path = entry.path();
                if path.read_dir()?.next().is_none() {
                    let _ = std::fs::remove_dir(&path);
                }
            }
        }
    }

    if !args.merged {
        return Ok(());
    }

    ui::log_step("Checking for spaces with merged PRs...");
    if std::process::Command::new("gh").arg("repo").arg("view").status().is_err() {
        bail!("GitHub CLI (gh) not found or not authenticated");
    }

    let main_branch = targets::current_branch(&repo_root).unwrap_or_else(|| "".to_string());
    let clone_dirs = list_clone_dirs(&clones_dir, &prefix)?;
    let mut removed = 0;
    let mut skipped = 0;

    for path in clone_dirs {
        let branch = targets::current_branch(&path).unwrap_or_else(|| "(detached)".to_string());
        let name = targets::space_name(&path, &prefix);
        if branch == "(detached)" || branch.is_empty() {
            skipped += 1;
            continue;
        }
        if branch == main_branch {
            continue;
        }

        let dirty = git::git_stdout_opt(["status", "--porcelain"], Some(&path))
            .map(|out| !out.trim().is_empty())
            .unwrap_or(false);
        if dirty {
            skipped += 1;
            continue;
        }

        let pr_state = gh_stdout_opt(
            [
                "pr",
                "list",
                "--head",
                &branch,
                "--state",
                "merged",
                "--json",
                "state",
                "--jq",
                ".[0].state",
            ],
            Some(&path),
        );
        if pr_state.as_deref() == Some("MERGED") {
            if args.dry_run {
                ui::log_info(&format!("[dry-run] Would remove: {name} ({})", path.display()));
                removed += 1;
            } else if args.yes || ui::prompt_yes_no(&format!("Remove space '{name}'?"), false)? {
                safe_remove_clone(&path, &clones_dir)?;
                removed += 1;
            } else {
                skipped += 1;
            }
        }
    }

    if args.dry_run {
        ui::log_info(&format!("Dry run complete. Would remove: {removed}, Skipped: {skipped}"));
    } else {
        ui::log_info(&format!("Merged cleanup complete. Removed: {removed}, Skipped: {skipped}"));
    }

    Ok(())
}

fn cmd_doctor() -> Result<()> {
    println!("Running spaces health check...");
    println!();

    if let Ok(version) = std::process::Command::new("git").arg("--version").output() {
        let text = String::from_utf8_lossy(&version.stdout);
        println!("[OK] Git: {}", text.trim());
    } else {
        println!("[x] Git: not found");
    }

    let repo_root = paths::repo_root()?;
    let clones_dir = paths::clones_dir(&repo_root)?;
    let mirror_dir = paths::mirror_dir(&repo_root)?;
    println!("[OK] Clones dir: {}", clones_dir.display());
    println!("[OK] Mirrors dir: {}", mirror_dir.display());
    println!(
        "[OK] Mirror present: {}",
        if mirror_dir.exists() { "yes" } else { "no" }
    );

    let default_branch = paths::default_branch(&repo_root)?;
    println!("[OK] Default branch: {default_branch}");

    Ok(())
}

fn cmd_mirrors(args: cli::MirrorsArgs) -> Result<()> {
    let repo_root = paths::repo_root()?;
    let mirror_dir = paths::mirror_dir(&repo_root)?;

    match args.command {
        Some(cli::MirrorsCommand::Update) => {
            mirror::ensure_mirror(&repo_root, &mirror_dir)?;
            mirror::update_mirror(&repo_root, &mirror_dir)?;
            println!("updated: {}", mirror_dir.display());
        }
        None => {
            println!("{}", mirror_dir.display());
            if mirror_dir.exists() {
                println!("status: present");
            } else {
                println!("status: missing");
            }
        }
    }
    Ok(())
}

fn cmd_config(args: cli::ConfigArgs) -> Result<()> {
    use config::Scope;

    let repo_root = paths::repo_root()?;
    let mut scope = Scope::Auto;
    let mut action: Option<String> = None;
    let mut key: Option<String> = None;
    let mut value: Option<String> = None;
    let mut extras = Vec::new();

    let mut iter = args.args.into_iter();
    while let Some(arg) = iter.next() {
        match arg.as_str() {
            "--global" | "global" => scope = Scope::Global,
            "--local" | "local" => scope = Scope::Local,
            "--system" | "system" => scope = Scope::System,
            "get" | "set" | "unset" | "add" | "list" => action = Some(arg),
            _ => {
                if key.is_none() {
                    key = Some(arg);
                } else if value.is_none() && matches!(action.as_deref(), Some("set") | Some("add")) {
                    value = Some(arg);
                } else {
                    extras.push(arg);
                }
            }
        }
    }

    let action = action.unwrap_or_else(|| if key.is_none() { "list".to_string() } else { "get".to_string() });

    if scope == Scope::System && matches!(action.as_str(), "set" | "add" | "unset") {
        bail!("--system not supported for write operations");
    }

    match action.as_str() {
        "get" => {
            let key = key.ok_or_else(|| anyhow::anyhow!("Usage: spaces config get <key> [--local|--global|--system]"))?;
            if !extras.is_empty() {
                ui::log_warn(&format!("get action: ignoring extra arguments: {}", extras.join(" ")));
            }
            let values = config::cfg_get_all(&key, scope, &repo_root);
            for value in values {
                println!("{value}");
            }
        }
        "set" => {
            let key = key.ok_or_else(|| anyhow::anyhow!("Usage: spaces config set <key> <value> [--local|--global]"))?;
            let value = value.ok_or_else(|| anyhow::anyhow!("Usage: spaces config set <key> <value> [--local|--global]"))?;
            if !extras.is_empty() {
                ui::log_warn(&format!("set action: ignoring extra arguments: {}", extras.join(" ")));
            }
            let resolved = if scope == Scope::Auto { Scope::Local } else { scope };
            config::cfg_set(&key, &value, resolved, &repo_root)?;
            ui::log_info(&format!("Config set: {key} = {value} ({:?})", resolved));
        }
        "add" => {
            let key = key.ok_or_else(|| anyhow::anyhow!("Usage: spaces config add <key> <value> [--local|--global]"))?;
            let value = value.ok_or_else(|| anyhow::anyhow!("Usage: spaces config add <key> <value> [--local|--global]"))?;
            if !extras.is_empty() {
                ui::log_warn(&format!("add action: ignoring extra arguments: {}", extras.join(" ")));
            }
            let resolved = if scope == Scope::Auto { Scope::Local } else { scope };
            config::cfg_add(&key, &value, resolved, &repo_root)?;
            ui::log_info(&format!("Config added: {key} = {value} ({:?})", resolved));
        }
        "unset" => {
            let key = key.ok_or_else(|| anyhow::anyhow!("Usage: spaces config unset <key> [--local|--global]"))?;
            if !extras.is_empty() || value.is_some() {
                ui::log_warn(&format!("unset action: ignoring extra arguments: {}", extras.join(" ")));
            }
            let resolved = if scope == Scope::Auto { Scope::Local } else { scope };
            config::cfg_unset(&key, resolved, &repo_root)?;
            ui::log_info(&format!("Config unset: {key} ({:?})", resolved));
        }
        "list" => {
            if key.is_some() || !extras.is_empty() {
                ui::log_warn("list action doesn't accept additional arguments (ignoring extras)");
            }
            let lines = config::cfg_list(scope, &repo_root)?;
            if lines.is_empty() {
                println!("No spaces configuration found");
            } else {
                for line in lines {
                    println!("{line}");
                }
            }
        }
        _ => {
            bail!("Unknown config action: {action}");
        }
    }

    Ok(())
}

fn safe_remove_clone(path: &Path, clones_dir: &Path) -> Result<()> {
    if !path.starts_with(clones_dir) {
        bail!("Refusing to remove path outside clones dir: {}", path.display());
    }
    if !path.join(".git").exists() {
        bail!("Refusing to remove non-git directory: {}", path.display());
    }
    std::fs::remove_dir_all(path)?;
    ui::log_info(&format!("Removed space: {}", path.display()));
    Ok(())
}

fn list_clone_dirs(clones_dir: &Path, prefix: &str) -> Result<Vec<PathBuf>> {
    if !clones_dir.is_dir() {
        return Ok(Vec::new());
    }
    let mut out = Vec::new();
    for entry in std::fs::read_dir(clones_dir)? {
        let entry = entry?;
        if !entry.file_type()?.is_dir() {
            continue;
        }
        let path = entry.path();
        if let Some(name) = path.file_name().and_then(|s| s.to_str()) {
            if !name.starts_with(prefix) {
                continue;
            }
        }
        out.push(path);
    }
    Ok(out)
}

fn list_space_names(clones_dir: &Path, prefix: &str) -> Result<Vec<String>> {
    let dirs = list_clone_dirs(clones_dir, prefix)?;
    let mut names = Vec::new();
    for path in dirs {
        names.push(targets::space_name(&path, prefix));
    }
    if names.is_empty() {
        bail!("No spaces found");
    }
    Ok(names)
}

fn gh_stdout_opt<I, S>(args: I, cwd: Option<&Path>) -> Option<String>
where
    I: IntoIterator<Item = S>,
    S: AsRef<std::ffi::OsStr>,
{
    let mut cmd = std::process::Command::new("gh");
    cmd.args(args);
    if let Some(dir) = cwd {
        cmd.current_dir(dir);
    }
    match cmd.output() {
        Ok(output) if output.status.success() => {
            let text = String::from_utf8_lossy(&output.stdout).trim().to_string();
            if text.is_empty() { None } else { Some(text) }
        }
        _ => None,
    }
}

fn dedupe(items: Vec<String>) -> Vec<String> {
    let mut seen = std::collections::HashSet::new();
    let mut out = Vec::new();
    for item in items {
        if seen.insert(item.clone()) {
            out.push(item);
        }
    }
    out
}
