use std::collections::BTreeMap;
use std::fs::{self, File};
use std::io::Write;
use std::os::unix::fs::PermissionsExt;
use std::path::PathBuf;

use anyhow::{Context, Result, bail};

use crate::agent::Agent;
use crate::paths::Paths;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Profile {
    values: BTreeMap<String, String>,
}

impl Profile {
    pub fn load(paths: &Paths, agent: Agent) -> Result<Self> {
        let file = paths.profile_file(agent);
        let iter = dotenvy::from_path_iter(&file)
            .with_context(|| format!("profile '{}' is not configured", agent.canonical()))?;
        let mut values = BTreeMap::new();
        for item in iter {
            let (key, value) =
                item.with_context(|| format!("invalid profile file {}", file.display()))?;
            values.insert(key, value);
        }
        let profile = Self { values };
        profile.validate(agent)?;
        Ok(profile)
    }

    pub fn value(&self, key: &str) -> Option<&str> {
        self.values.get(key).map(String::as_str)
    }

    pub fn iter(&self) -> impl Iterator<Item = (&String, &String)> {
        self.values.iter()
    }

    fn validate(&self, agent: Agent) -> Result<()> {
        self.require(agent, "CLAUDE_CONFIG_DIR")?;
        match agent {
            Agent::Max => {}
            Agent::Api => self.require(agent, "ANTHROPIC_API_KEY")?,
            Agent::Glm | Agent::Mimo | Agent::Deepseek | Agent::Kimi => {
                self.require(agent, "ANTHROPIC_BASE_URL")?;
                self.require(agent, "ANTHROPIC_AUTH_TOKEN")?;
            }
        }
        Ok(())
    }

    fn require(&self, agent: Agent, key: &str) -> Result<()> {
        match self.value(key) {
            Some(value) if !value.is_empty() => Ok(()),
            _ => bail!("profile '{}' must define {key}", agent.canonical()),
        }
    }
}

pub fn write_template(paths: &Paths, agent: Agent) -> Result<PathBuf> {
    fs::create_dir_all(paths.profiles_dir())?;
    let file = paths.profile_file(agent);
    if file.exists() {
        return Ok(file);
    }
    let mut handle = File::create(&file)?;
    for (key, value) in agent.template(paths.home()) {
        writeln!(handle, "{key}={value}")?;
    }
    let mut permissions = handle.metadata()?.permissions();
    permissions.set_mode(0o600);
    fs::set_permissions(&file, permissions)?;
    Ok(file)
}

pub fn read_default_profile(paths: &Paths) -> Result<Option<Agent>> {
    let file = paths.config_file();
    if !file.exists() {
        return Ok(None);
    }
    let content = fs::read_to_string(file)?;
    for line in content.lines() {
        if let Some(value) = line.strip_prefix("default_profile=") {
            return Ok(Some(Agent::parse(value)?));
        }
    }
    Ok(None)
}

pub fn write_default_profile(paths: &Paths, agent: Agent) -> Result<()> {
    fs::create_dir_all(paths.ccs_home())?;
    fs::write(
        paths.config_file(),
        format!("default_profile={}\n", agent.canonical()),
    )?;
    Ok(())
}
