use std::ffi::OsStr;
use std::path::Path;
use std::process::{Command, Output};

use anyhow::{bail, Context, Result};

pub fn git_output<I, S>(args: I, cwd: Option<&Path>) -> Result<Output>
where
    I: IntoIterator<Item = S>,
    S: AsRef<OsStr>,
{
    let mut cmd = Command::new("git");
    cmd.args(args);
    if let Some(dir) = cwd {
        cmd.current_dir(dir);
    }
    cmd.output().context("failed to run git")
}

pub fn git_check<I, S>(args: I, cwd: Option<&Path>) -> Result<()>
where
    I: IntoIterator<Item = S>,
    S: AsRef<OsStr>,
{
    let output = git_output(args, cwd)?;
    if output.status.success() {
        Ok(())
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr);
        bail!(stderr.trim().to_string())
    }
}

pub fn git_stdout<I, S>(args: I, cwd: Option<&Path>) -> Result<String>
where
    I: IntoIterator<Item = S>,
    S: AsRef<OsStr>,
{
    let output = git_output(args, cwd)?;
    if output.status.success() {
        Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr);
        bail!(stderr.trim().to_string())
    }
}

pub fn git_stdout_opt<I, S>(args: I, cwd: Option<&Path>) -> Option<String>
where
    I: IntoIterator<Item = S>,
    S: AsRef<OsStr>,
{
    match git_output(args, cwd) {
        Ok(output) if output.status.success() => {
            let out = String::from_utf8_lossy(&output.stdout).trim().to_string();
            if out.is_empty() {
                None
            } else {
                Some(out)
            }
        }
        _ => None,
    }
}
