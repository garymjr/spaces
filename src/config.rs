use std::collections::HashSet;
use std::path::{Path, PathBuf};

use anyhow::Result;

use crate::git;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Scope {
    Auto,
    Local,
    Global,
    System,
}

pub fn scope_flag(scope: Scope) -> &'static str {
    match scope {
        Scope::Auto => "",
        Scope::Local => "--local",
        Scope::Global => "--global",
        Scope::System => "--system",
    }
}

pub fn spacesrc_path(repo_root: &Path) -> PathBuf {
    repo_root.join(".spacesrc")
}

fn git_config_get(key: &str, scope: Scope, repo_root: &Path) -> Option<String> {
    let mut args = vec!["config"]; 
    let flag = scope_flag(scope);
    if !flag.is_empty() {
        args.push(flag);
    }
    args.push("--get");
    args.push(key);
    git::git_stdout_opt(args, Some(repo_root))
}

fn git_config_get_all(key: &str, scope: Scope, repo_root: &Path) -> Vec<String> {
    let mut args = vec!["config"]; 
    let flag = scope_flag(scope);
    if !flag.is_empty() {
        args.push(flag);
    }
    args.push("--get-all");
    args.push(key);
    match git::git_stdout_opt(args, Some(repo_root)) {
        Some(value) => value.lines().map(|line| line.to_string()).collect(),
        None => Vec::new(),
    }
}

fn git_config_file_get_all(key: &str, repo_root: &Path) -> Vec<String> {
    let file = spacesrc_path(repo_root);
    if !file.exists() {
        return Vec::new();
    }
    let file_str = file.to_string_lossy().to_string();
    let args = ["config", "-f", &file_str, "--get-all", key];
    match git::git_stdout_opt(args, Some(repo_root)) {
        Some(value) => value.lines().map(|line| line.to_string()).collect(),
        None => Vec::new(),
    }
}

pub fn cfg_get_all(key: &str, scope: Scope, repo_root: &Path) -> Vec<String> {
    match scope {
        Scope::Local | Scope::Global | Scope::System => git_config_get_all(key, scope, repo_root),
        Scope::Auto => {
            let mut seen = HashSet::new();
            let mut out = Vec::new();

            for value in git_config_get_all(key, Scope::Local, repo_root) {
                if seen.insert(value.clone()) {
                    out.push(value);
                }
            }
            for value in git_config_file_get_all(key, repo_root) {
                if seen.insert(value.clone()) {
                    out.push(value);
                }
            }
            for value in git_config_get_all(key, Scope::Global, repo_root) {
                if seen.insert(value.clone()) {
                    out.push(value);
                }
            }
            for value in git_config_get_all(key, Scope::System, repo_root) {
                if seen.insert(value.clone()) {
                    out.push(value);
                }
            }

            out
        }
    }
}

pub fn cfg_default(
    key: &str,
    env_name: &str,
    fallback: &str,
    file_key: Option<&str>,
    repo_root: &Path,
) -> Result<String> {
    if let Some(value) = git_config_get(key, Scope::Local, repo_root) {
        if !value.is_empty() {
            return Ok(value);
        }
    }

    if let Some(file_key) = file_key {
        let values = git_config_file_get_all(file_key, repo_root);
        if let Some(first) = values.first() {
            if !first.is_empty() {
                return Ok(first.clone());
            }
        }
    } else {
        let values = git_config_file_get_all(key, repo_root);
        if let Some(first) = values.first() {
            if !first.is_empty() {
                return Ok(first.clone());
            }
        }
    }

    if let Some(value) = git_config_get(key, Scope::Auto, repo_root) {
        if !value.is_empty() {
            return Ok(value);
        }
    }

    if !env_name.is_empty() {
        if let Ok(value) = std::env::var(env_name) {
            if !value.is_empty() {
                return Ok(value);
            }
        }
    }

    Ok(fallback.to_string())
}

pub fn cfg_set(key: &str, value: &str, scope: Scope, repo_root: &Path) -> Result<()> {
    let mut args = vec!["config"]; 
    let flag = scope_flag(scope);
    if !flag.is_empty() {
        args.push(flag);
    }
    args.push(key);
    args.push(value);
    git::git_check(args, Some(repo_root))
}

pub fn cfg_add(key: &str, value: &str, scope: Scope, repo_root: &Path) -> Result<()> {
    let mut args = vec!["config"]; 
    let flag = scope_flag(scope);
    if !flag.is_empty() {
        args.push(flag);
    }
    args.push("--add");
    args.push(key);
    args.push(value);
    git::git_check(args, Some(repo_root))
}

pub fn cfg_unset(key: &str, scope: Scope, repo_root: &Path) -> Result<()> {
    let mut args = vec!["config"]; 
    let flag = scope_flag(scope);
    if !flag.is_empty() {
        args.push(flag);
    }
    args.push("--unset-all");
    args.push(key);
    let _ = git::git_check(args, Some(repo_root));
    Ok(())
}

pub fn cfg_list(scope: Scope, repo_root: &Path) -> Result<Vec<String>> {
    let mut lines = Vec::new();

    match scope {
        Scope::Local | Scope::Global | Scope::System => {
            let mut args = vec!["config"]; 
            let flag = scope_flag(scope);
            if !flag.is_empty() {
                args.push(flag);
            }
            args.push("--get-regexp");
            args.push("^spaces\\.");
            if let Some(output) = git::git_stdout_opt(args, Some(repo_root)) {
                for line in output.lines() {
                    lines.push(line.to_string());
                }
            }
        }
        Scope::Auto => {
            let mut seen = HashSet::new();

            let sources = [
                (Scope::Local, "local"),
                (Scope::Global, "global"),
                (Scope::System, "system"),
            ];

            let local_lines = git::git_stdout_opt(
                ["config", "--local", "--get-regexp", "^spaces\\."],
                Some(repo_root),
            );
            if let Some(output) = local_lines {
                for line in output.lines() {
                    if seen.insert(line.to_string()) {
                        lines.push(format!("{line} [local]"));
                    }
                }
            }

            let file = spacesrc_path(repo_root);
            if file.exists() {
                let file_str = file.to_string_lossy().to_string();
                let args = ["config", "-f", &file_str, "--get-regexp", "^spaces\\."]; 
                if let Some(output) = git::git_stdout_opt(args, Some(repo_root)) {
                    for line in output.lines() {
                        if seen.insert(line.to_string()) {
                            lines.push(format!("{line} [.spacesrc]"));
                        }
                    }
                }
            }

            for (scope, label) in sources {
                if scope == Scope::Local {
                    continue;
                }
                let mut args = vec!["config"]; 
                args.push(scope_flag(scope));
                args.push("--get-regexp");
                args.push("^spaces\\.");
                if let Some(output) = git::git_stdout_opt(args, Some(repo_root)) {
                    for line in output.lines() {
                        if seen.insert(line.to_string()) {
                            lines.push(format!("{line} [{label}]"));
                        }
                    }
                }
            }
        }
    }

    Ok(lines)
}
