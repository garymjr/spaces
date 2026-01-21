use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{bail, Context, Result};

use crate::git;
use crate::ui;

pub struct ClonePlan {
    pub path: PathBuf,
    pub branch: Option<String>,
    pub base_ref: String,
}

pub fn create_clone(repo_root: &Path, mirror_dir: &Path, plan: &ClonePlan) -> Result<()> {
    if plan.path.exists() {
        bail!("Clone already exists: {}", plan.path.display());
    }

    if let Some(parent) = plan.path.parent() {
        fs::create_dir_all(parent).with_context(|| format!("create clones dir {parent:?}"))?;
    }

    let clone_source = git::git_stdout_opt(["remote", "get-url", "origin"], Some(repo_root))
        .unwrap_or_else(|| repo_root.to_string_lossy().to_string());

    ui::log_step("Cloning repository...");
    git::git_check(
        [
            "clone",
            "--reference-if-able",
            mirror_dir.to_string_lossy().as_ref(),
            clone_source.as_str(),
            plan.path.to_string_lossy().as_ref(),
        ],
        None,
    )?;

    if let Some(branch) = &plan.branch {
        checkout_branch(mirror_dir, plan, branch)?;
    }
    Ok(())
}

fn checkout_branch(mirror_dir: &Path, plan: &ClonePlan, branch: &str) -> Result<()> {
    let remote_ref = format!("refs/remotes/origin/{branch}");
    let local_ref = format!("refs/heads/{branch}");

    if git::git_check(["show-ref", "--verify", "--quiet", &remote_ref], Some(mirror_dir)).is_ok()
    {
        git::git_check(
            ["checkout", "-b", branch, &format!("origin/{branch}")],
            Some(&plan.path),
        )?;
        return Ok(());
    }

    if git::git_check(["show-ref", "--verify", "--quiet", &local_ref], Some(mirror_dir)).is_ok() {
        git::git_check(["checkout", branch], Some(&plan.path))?;
        return Ok(());
    }

    git::git_check(["checkout", "-b", branch, &plan.base_ref], Some(&plan.path))?;
    Ok(())
}
