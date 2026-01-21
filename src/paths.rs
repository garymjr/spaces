use std::path::{Path, PathBuf};

use anyhow::{Context, Result};

use crate::config;
use crate::git;

pub fn repo_root() -> Result<PathBuf> {
    let root = git::git_stdout(["rev-parse", "--show-toplevel"], None)?;
    Ok(PathBuf::from(root))
}

pub fn sanitize_branch_name(name: &str) -> String {
    let mut out = String::with_capacity(name.len());
    for ch in name.chars() {
        let replacement = match ch {
            '/' | '\\' | ' ' | ':' | '*' | '?' | '"' | '<' | '>' | '|' | '#' => '-',
            _ => ch,
        };
        out.push(replacement);
    }
    out.trim_matches('-').to_string()
}

pub fn expand_home(path: &str) -> PathBuf {
    if path == "~" {
        return dirs::home_dir().unwrap_or_else(|| PathBuf::from("~"));
    }
    if let Some(stripped) = path.strip_prefix("~/") {
        let home = dirs::home_dir().unwrap_or_else(|| PathBuf::from("~"));
        return home.join(stripped);
    }
    PathBuf::from(path)
}

pub fn clones_dir(repo_root: &Path) -> Result<PathBuf> {
    let configured = config::cfg_default("spaces.clones.dir", "SPACES_CLONES_DIR", "", None, repo_root)?;
    if !configured.is_empty() {
        let mut dir = expand_home(&configured);
        if dir.is_relative() {
            dir = repo_root.join(dir);
        }
        return Ok(dir);
    }

    let repo_name = repo_root
        .file_name()
        .and_then(|s| s.to_str())
        .unwrap_or("repo");
    let parent = repo_root
        .parent()
        .context("repo has no parent")?;
    Ok(parent.join(format!("{repo_name}-clones")))
}

pub fn clones_prefix(repo_root: &Path) -> Result<String> {
    Ok(config::cfg_default("spaces.clones.prefix", "SPACES_CLONES_PREFIX", "", None, repo_root)?)
}

pub fn mirror_dir(repo_root: &Path) -> Result<PathBuf> {
    let configured = config::cfg_default("spaces.mirrors.dir", "SPACES_MIRRORS_DIR", "", None, repo_root)?;
    if !configured.is_empty() {
        let mut dir = expand_home(&configured);
        if dir.is_relative() {
            dir = repo_root.join(dir);
        }
        return Ok(dir);
    }

    let repo_name = repo_root
        .file_name()
        .and_then(|s| s.to_str())
        .unwrap_or("repo");
    let mut base = dirs::home_dir().unwrap_or_else(|| PathBuf::from("~"));
    base.push(".cache");
    base.push("spaces");
    base.push("mirrors");
    base.push(repo_name);
    Ok(base)
}

pub fn default_branch(repo_root: &Path) -> Result<String> {
    let configured = config::cfg_default("spaces.defaultBranch", "SPACES_DEFAULT_BRANCH", "auto", None, repo_root)?;
    if configured != "auto" {
        return Ok(configured);
    }

    if let Some(origin_head) = git::git_stdout_opt(["symbolic-ref", "--quiet", "refs/remotes/origin/HEAD"], Some(repo_root)) {
        if let Some(stripped) = origin_head.strip_prefix("refs/remotes/origin/") {
            return Ok(stripped.to_string());
        }
    }

    if git::git_check(["show-ref", "--verify", "--quiet", "refs/remotes/origin/main"], Some(repo_root)).is_ok() {
        return Ok("main".to_string());
    }
    if git::git_check(["show-ref", "--verify", "--quiet", "refs/remotes/origin/master"], Some(repo_root)).is_ok() {
        return Ok("master".to_string());
    }

    Ok("main".to_string())
}
