use std::collections::HashMap;
use std::path::Path;
use std::process::Command;

use anyhow::{bail, Result};

use crate::config;
use crate::ui;

pub fn run_hooks(phase: &str, repo_root: &Path, cwd: &Path, envs: &HashMap<String, String>) -> Result<()> {
    let key = format!("spaces.hook.{phase}");
    let hooks = config::cfg_get_all(&key, config::Scope::Auto, repo_root);
    if hooks.is_empty() {
        return Ok(());
    }

    ui::log_step(&format!("Running {phase} hooks..."));
    let mut failed = 0;

    for (idx, hook) in hooks.iter().enumerate() {
        if hook.trim().is_empty() {
            continue;
        }
        ui::log_info(&format!("Hook {}: {}", idx + 1, hook));
        let mut cmd = Command::new("sh");
        cmd.arg("-c").arg(hook).current_dir(cwd);
        for (k, v) in envs {
            cmd.env(k, v);
        }
        let status = cmd.status()?;
        if !status.success() {
            failed += 1;
            ui::log_error(&format!("Hook {} failed", idx + 1));
        }
    }

    if failed > 0 {
        bail!("{failed} hook(s) failed")
    } else {
        Ok(())
    }
}
