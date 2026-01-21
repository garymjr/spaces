use std::fs;
use std::path::Path;

use anyhow::{Context, Result};

use crate::git;
use crate::ui;

pub fn ensure_mirror(repo_root: &Path, mirror_dir: &Path) -> Result<()> {
    if mirror_dir.exists() {
        return Ok(());
    }

    if let Some(parent) = mirror_dir.parent() {
        fs::create_dir_all(parent).with_context(|| format!("create mirror parent {parent:?}"))?;
    }

    ui::log_step(&format!("Creating mirror: {}", mirror_dir.display()));
    git::git_check([
        "clone",
        "--mirror",
        repo_root.to_string_lossy().as_ref(),
        mirror_dir.to_string_lossy().as_ref(),
    ], None)
}

pub fn update_mirror(repo_root: &Path, mirror_dir: &Path) -> Result<()> {
    let origin_url = git::git_stdout_opt(["remote", "get-url", "origin"], Some(repo_root));
    if let Some(url) = origin_url {
        let _ = git::git_check(["remote", "set-url", "origin", &url], Some(mirror_dir));
        let _ = git::git_check(["fetch", "--prune", "origin"], Some(mirror_dir));
    }

    let repo_root_str = repo_root.to_string_lossy();
    let fetch_args = [
        "fetch",
        repo_root_str.as_ref(),
        "+refs/heads/*:refs/heads/*",
        "+refs/tags/*:refs/tags/*",
    ];
    let _ = git::git_check(fetch_args, Some(mirror_dir));

    Ok(())
}
