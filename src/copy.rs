use std::collections::HashSet;
use std::fs;
use std::path::Path;

use anyhow::{Context, Result};
use glob::{glob_with, MatchOptions, Pattern};
use walkdir::WalkDir;

use crate::ui;

pub fn parse_pattern_file(path: &Path) -> Result<Vec<String>> {
    if !path.exists() {
        return Ok(Vec::new());
    }
    let content = fs::read_to_string(path).with_context(|| format!("read pattern file {}", path.display()))?;
    let mut out = Vec::new();
    for line in content.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() || trimmed.starts_with('#') {
            continue;
        }
        out.push(trimmed.to_string());
    }
    Ok(out)
}

fn is_unsafe_pattern(pattern: &str) -> bool {
    pattern.starts_with('/')
        || pattern == ".."
        || pattern.starts_with("../")
        || pattern.contains("/../")
        || pattern.ends_with("/..")
}

pub fn copy_patterns(
    src_root: &Path,
    dst_root: &Path,
    includes: &[String],
    excludes: &[String],
    dry_run: bool,
) -> Result<()> {
    if includes.is_empty() {
        return Ok(());
    }

    let mut exclude_patterns = Vec::new();
    for pattern in excludes {
        if is_unsafe_pattern(pattern) {
            ui::log_warn(&format!("Skipping unsafe exclude pattern: {pattern}"));
            continue;
        }
        if let Ok(pat) = Pattern::new(pattern) {
            exclude_patterns.push(pat);
        }
    }

    let options = MatchOptions {
        case_sensitive: true,
        require_literal_separator: false,
        require_literal_leading_dot: false,
    };

    let mut copied = 0;
    let mut seen = HashSet::new();

    for pattern in includes {
        if is_unsafe_pattern(pattern) {
            ui::log_warn(&format!("Skipping unsafe pattern: {pattern}"));
            continue;
        }

        let normalized = pattern.strip_prefix("./").unwrap_or(pattern);
        let full = src_root.join(normalized);
        let full_pattern = full.to_string_lossy().to_string();

        for entry in glob_with(&full_pattern, options)? {
            let path = match entry {
                Ok(p) => p,
                Err(_) => continue,
            };
            if !path.is_file() {
                continue;
            }
            let rel = match path.strip_prefix(src_root) {
                Ok(rel) => rel,
                Err(_) => continue,
            };
            let rel_str = rel.to_string_lossy();

            if exclude_patterns.iter().any(|pat| pat.matches_path_with(rel, options)) {
                continue;
            }

            if !seen.insert(rel.to_path_buf()) {
                continue;
            }

            let dest = dst_root.join(rel);
            if dry_run {
                ui::log_info(&format!("[dry-run] Would copy: {rel_str}"));
            } else {
                if let Some(parent) = dest.parent() {
                    fs::create_dir_all(parent)?;
                }
                fs::copy(&path, &dest)?;
                ui::log_info(&format!("Copied {rel_str}"));
            }
            copied += 1;
        }
    }

    if copied > 0 {
        if dry_run {
            ui::log_info(&format!("[dry-run] Would copy {copied} file(s)"));
        } else {
            ui::log_info(&format!("Copied {copied} file(s)"));
        }
    }

    Ok(())
}

pub fn copy_directories(
    src_root: &Path,
    dst_root: &Path,
    includes: &[String],
    excludes: &[String],
) -> Result<()> {
    if includes.is_empty() {
        return Ok(());
    }

    let mut exclude_patterns = Vec::new();
    for pattern in excludes {
        if is_unsafe_pattern(pattern) {
            ui::log_warn(&format!("Skipping unsafe exclude pattern: {pattern}"));
            continue;
        }
        if let Ok(pat) = Pattern::new(pattern) {
            exclude_patterns.push(pat);
        }
    }

    let mut copied = 0;

    for pattern in includes {
        if is_unsafe_pattern(pattern) {
            ui::log_warn(&format!("Skipping unsafe pattern: {pattern}"));
            continue;
        }
        let matcher = match Pattern::new(pattern) {
            Ok(p) => p,
            Err(_) => continue,
        };

        for entry in WalkDir::new(src_root).min_depth(1).into_iter().filter_map(Result::ok) {
            if !entry.file_type().is_dir() {
                continue;
            }
            let name = entry.file_name().to_string_lossy();
            if !matcher.matches(&name) {
                continue;
            }

            let rel = match entry.path().strip_prefix(src_root) {
                Ok(rel) => rel,
                Err(_) => continue,
            };
            let rel_str = rel.to_string_lossy();

            if exclude_patterns
                .iter()
                .any(|pat| pat.matches_path_with(rel, MatchOptions { case_sensitive: true, require_literal_separator: false, require_literal_leading_dot: false }))
            {
                continue;
            }

            let dest_dir = dst_root.join(rel);
            copy_dir_recursive(entry.path(), &dest_dir, &exclude_patterns)?;
            ui::log_info(&format!("Copied directory {rel_str}"));
            copied += 1;
        }
    }

    if copied > 0 {
        ui::log_info(&format!("Copied {copied} directories"));
    }

    Ok(())
}

fn copy_dir_recursive(src: &Path, dst: &Path, excludes: &[Pattern]) -> Result<()> {
    for entry in WalkDir::new(src).into_iter().filter_map(Result::ok) {
        let path = entry.path();
        let rel = match path.strip_prefix(src) {
            Ok(rel) => rel,
            Err(_) => continue,
        };
        if rel.as_os_str().is_empty() {
            continue;
        }
        if excludes.iter().any(|pat| pat.matches_path(rel)) {
            continue;
        }

        let target = dst.join(rel);
        if entry.file_type().is_dir() {
            fs::create_dir_all(&target)?;
        } else if entry.file_type().is_file() {
            if let Some(parent) = target.parent() {
                fs::create_dir_all(parent)?;
            }
            fs::copy(path, &target)?;
        } else if entry.file_type().is_symlink() {
            #[cfg(unix)]
            {
                use std::os::unix::fs as unix_fs;
                let link = fs::read_link(path)?;
                if let Some(parent) = target.parent() {
                    fs::create_dir_all(parent)?;
                }
                let _ = unix_fs::symlink(link, &target);
            }
        }
    }
    Ok(())
}
