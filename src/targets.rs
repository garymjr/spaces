use std::path::{Path, PathBuf};

use anyhow::{bail, Result};

use crate::git;
use crate::paths;

pub struct Target {
    pub is_main: bool,
    pub path: PathBuf,
    pub name: String,
    pub branch: String,
}

pub fn current_branch(path: &Path) -> Option<String> {
    let branch = git::git_stdout_opt(["branch", "--show-current"], Some(path))
        .or_else(|| git::git_stdout_opt(["rev-parse", "--abbrev-ref", "HEAD"], Some(path)));
    match branch {
        Some(b) if b == "HEAD" => Some("(detached)".to_string()),
        Some(b) if !b.is_empty() => Some(b),
        _ => None,
    }
}

pub fn resolve_target(identifier: &str, repo_root: &Path, clones_dir: &Path, prefix: &str) -> Result<Target> {
    if identifier == "1" {
        let branch = current_branch(repo_root).unwrap_or_else(|| "(detached)".to_string());
        return Ok(Target {
            is_main: true,
            path: repo_root.to_path_buf(),
            name: "main".to_string(),
            branch,
        });
    }

    let sanitized = paths::sanitize_branch_name(identifier);
    let direct = clones_dir.join(format!("{prefix}{sanitized}"));
    if direct.is_dir() {
        let branch = current_branch(&direct).unwrap_or_else(|| "(detached)".to_string());
        return Ok(Target {
            is_main: false,
            name: identifier.to_string(),
            path: direct,
            branch,
        });
    }

    bail!("Target not found for space: {identifier}")
}

pub fn status(path: &Path) -> String {
    if !path.exists() {
        return "missing".to_string();
    }
    let branch = current_branch(path).unwrap_or_else(|| "(detached)".to_string());
    if branch == "(detached)" {
        return "detached".to_string();
    }

    if let Some(output) = git::git_stdout_opt(["status", "--porcelain"], Some(path)) {
        if !output.trim().is_empty() {
            return "dirty".to_string();
        }
    }

    "ok".to_string()
}

pub fn space_name(path: &Path, prefix: &str) -> String {
    let name = path
        .file_name()
        .and_then(|s| s.to_str())
        .unwrap_or("space");
    name.strip_prefix(prefix).unwrap_or(name).to_string()
}
