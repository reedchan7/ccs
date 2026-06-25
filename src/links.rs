use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result};

use crate::profile::Profile;

const DEFAULT_SHARED_PATHS: &str = "CLAUDE.md,settings.json,skills,plugins,rules";

pub fn ensure_shared_links(profile: &Profile) -> Result<()> {
    let Some(config_dir) = profile.value("CLAUDE_CONFIG_DIR") else {
        return Ok(());
    };
    let config_dir = PathBuf::from(config_dir);
    let shared_dir = profile
        .value("CCS_SHARED_CLAUDE_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(|| dirs_home().join(".claude"));

    if !shared_dir.exists() {
        return Ok(());
    }

    fs::create_dir_all(&config_dir)?;
    let shared_paths = profile
        .value("CCS_SHARED_PATHS")
        .unwrap_or(DEFAULT_SHARED_PATHS);

    for name in shared_paths
        .split(',')
        .map(str::trim)
        .filter(|value| !value.is_empty())
    {
        let source = shared_dir.join(name);
        if !source.exists() {
            continue;
        }

        let target = config_dir.join(name);
        if is_symlink_to(&target, &source)? {
            continue;
        }

        if target.exists() || target.is_symlink() {
            backup_existing(&config_dir, name, &target)?;
        }

        if let Some(parent) = target.parent() {
            fs::create_dir_all(parent)?;
        }

        std::os::unix::fs::symlink(&source, &target)
            .with_context(|| format!("link {} -> {}", target.display(), source.display()))?;
    }

    Ok(())
}

fn dirs_home() -> PathBuf {
    directories::BaseDirs::new()
        .map(|dirs| dirs.home_dir().to_path_buf())
        .unwrap_or_else(|| PathBuf::from("."))
}

fn is_symlink_to(target: &Path, source: &Path) -> Result<bool> {
    if !target.is_symlink() {
        return Ok(false);
    }
    Ok(fs::read_link(target)? == source)
}

fn backup_existing(config_dir: &Path, name: &str, target: &Path) -> Result<()> {
    let backup_root = config_dir.join(".ccs-local-backup");
    let mut backup = backup_root.join(name);
    let mut suffix = 1;
    while backup.exists() || backup.is_symlink() {
        backup = backup_root.join(format!("{name}.bak.{suffix}"));
        suffix += 1;
    }
    if let Some(parent) = backup.parent() {
        fs::create_dir_all(parent)?;
    }
    fs::rename(target, backup)?;
    Ok(())
}
