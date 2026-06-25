use std::path::{Path, PathBuf};

use crate::agent::Agent;

#[derive(Debug, Clone)]
pub struct Paths {
    home: PathBuf,
    ccs_home: PathBuf,
}

impl Paths {
    pub fn from_home(home: impl AsRef<Path>) -> Self {
        let home = home.as_ref().to_path_buf();
        let ccs_home = home.join(".config").join("ccs");
        Self { home, ccs_home }
    }

    pub fn from_env() -> anyhow::Result<Self> {
        if let Some(home) = std::env::var_os("CCS_TEST_HOME") {
            return Ok(Self::from_home(home));
        }
        let dirs = directories::BaseDirs::new()
            .ok_or_else(|| anyhow::anyhow!("could not resolve home directory"))?;
        Ok(Self::from_home(dirs.home_dir()))
    }

    pub fn home(&self) -> &Path {
        &self.home
    }

    pub fn ccs_home(&self) -> &Path {
        &self.ccs_home
    }

    pub fn config_file(&self) -> PathBuf {
        self.ccs_home.join("config")
    }

    pub fn profiles_dir(&self) -> PathBuf {
        self.ccs_home.join("profiles")
    }

    pub fn profile_file(&self, agent: Agent) -> PathBuf {
        self.profiles_dir()
            .join(format!("{}.env", agent.canonical()))
    }

    pub fn claude_dir(&self, agent: Agent) -> PathBuf {
        self.ccs_home.join("claude").join(agent.canonical())
    }

    pub fn hook_file(&self) -> PathBuf {
        self.ccs_home.join("ccs.sh")
    }
}
