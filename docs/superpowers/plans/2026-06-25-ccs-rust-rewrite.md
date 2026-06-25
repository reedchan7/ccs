# ccs Rust Rewrite Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Rewrite `ccs` as a Rust Edition 2024 CLI whose shortest daily paths are `ccs`, `ccs ds`, and `ccs use ds`.

**Architecture:** Keep the runtime model file-based and local: dotenv profile files under `~/.config/ccs/profiles`, a key-value global config, and profile-specific Claude config dirs. Use a manual top-level dispatcher for `ccs <agent> [claude args...]`, then use typed command functions for management commands. Keep shell integration thin: the shell function only handles parent-shell env mutation for `ccs use <agent>`.

**Tech Stack:** Rust 2024, clap, anyhow, serde_json, directories, dotenvy, tempfile, assert_cmd, predicates, self_update or direct GitHub Release download.

## Global Constraints

- Keep `ccs` as the installed executable name.
- Make `ccs` with no arguments enter the default Claude Code agent.
- Make `ccs <agent>` enter a specific Claude Code agent directly.
- Support `ds` as a stable alias for `deepseek` everywhere an agent name is accepted.
- Rebuild the implementation in Rust Edition 2024.
- Preserve the existing goal: session-level Claude Code use with multiple Anthropic-compatible models.
- Keep the existing on-disk profile data usable under `~/.config/ccs`.
- Add self-update from GitHub Releases.
- Add formatting, linting, Makefile targets, CI, and release builds for Linux x86_64, Linux aarch64, macOS x86_64, and macOS aarch64.
- Do not print profile secrets in status or profile listing output.
- Keep `README.md` and `bin/ccs` user changes intact until the task that intentionally replaces them.

---

## Spec Review Notes

- The spec is coherent after two small edits: `ccs profiles list` is now listed with `ccs profiles` and `ccs profiles ls`, and the hook snippet now routes `ccs use <agent> --global` to the Rust binary instead of env eval.
- Keep the first Rust version on the fixed built-in agent set: `max`, `api`, `glm`, `mimo`, `deepseek`, `kimi`, with alias `ds`.
- Implement `ccs profiles add <agent>` for built-in agents only. Custom profile names can be added later if there is real demand.
- Treat exact `exec` as a Unix production path, but expose a test mode that spawns fake `claude` and returns its exit code.

## File Structure

- Create: `Cargo.toml` - package metadata, Rust 2024 edition, dependencies.
- Create: `Cargo.lock` - locked dependency graph.
- Create: `src/lib.rs` - module exports used by integration tests.
- Create: `src/main.rs` - binary entrypoint and error printing.
- Create: `src/cli.rs` - top-level argument dispatch and management command parsing.
- Create: `src/agent.rs` - canonical agent names, aliases, built-in profile templates.
- Create: `src/paths.rs` - `~/.config/ccs` path resolution with test override support.
- Create: `src/profile.rs` - dotenv profile loading, writing, and validation.
- Create: `src/env.rs` - env payload creation, shell export rendering, and known-var clearing.
- Create: `src/links.rs` - shared Claude config symlink handling and local backup.
- Create: `src/run.rs` - launching Claude with an agent env.
- Create: `src/shell.rs` - hook script rendering and installation.
- Create: `src/permissions.rs` - `.claude/settings.local.json` mutation.
- Create: `src/update.rs` - release asset naming and self-update.
- Create: `tests/support/mod.rs` - temp home, fake `claude`, and command helpers.
- Create: `tests/cli.rs` - CLI parsing and command behavior tests.
- Create: `tests/profiles.rs` - profile/env/shared-link behavior tests.
- Create: `tests/launch.rs` - `ccs`, `ccs ds`, and arg passthrough tests.
- Create: `tests/shell.rs` - `ccs use`, hook, init behavior tests.
- Create: `tests/permissions.rs` - permissions JSON behavior tests.
- Create: `tests/update.rs` - platform asset naming tests.
- Modify: `install.sh` - install compiled binary and run hook setup.
- Replace: `bin/ccs` - thin shim to `target/release/ccs` for local dev or remove from install path once Rust binary is canonical.
- Modify: `README.md` - document new daily workflow.
- Create: `Makefile` - fmt, lint, test, build, install, release-local.
- Create: `.github/workflows/ci.yml` - Rust CI.
- Create: `.github/workflows/release.yml` - tag release binary build.

## Task 1: Rust Project Skeleton and Top-Level CLI Dispatch

**Files:**
- Create: `Cargo.toml`
- Create: `src/lib.rs`
- Create: `src/main.rs`
- Create: `src/cli.rs`
- Create: `tests/cli.rs`

**Interfaces:**
- Produces: `ccs::cli::parse<I, S>(args: I) -> anyhow::Result<Command>` where `I: IntoIterator<Item = S>` and `S: Into<std::ffi::OsString>`.
- Produces: `ccs::cli::Command` with variants `LaunchDefault`, `LaunchAgent`, `Use`, `Profiles`, `Init`, `Status`, `Update`, `PermissionsBypass`, `InternalEnv`, and `Help`.
- Consumes: no prior task.

- [ ] **Step 1: Add the first parser tests**

Create `tests/cli.rs`:

```rust
use std::ffi::OsString;

use ccs::cli::{parse, Command, ProfilesCommand};

fn os_vec(items: &[&str]) -> Vec<OsString> {
    items.iter().map(OsString::from).collect()
}

#[test]
fn bare_ccs_launches_default_agent() {
    let command = parse(os_vec(&["ccs"])).unwrap();
    assert_eq!(command, Command::LaunchDefault);
}

#[test]
fn first_unknown_token_is_agent_entry_with_passthrough_args() {
    let command = parse(os_vec(&["ccs", "ds", "--print", "hello"])).unwrap();
    assert_eq!(
        command,
        Command::LaunchAgent {
            agent: "ds".into(),
            claude_args: os_vec(&["--print", "hello"]),
        }
    );
}

#[test]
fn use_supports_global_scope() {
    let command = parse(os_vec(&["ccs", "use", "ds", "--global"])).unwrap();
    assert_eq!(
        command,
        Command::Use {
            agent: "ds".into(),
            global: true,
        }
    );
}

#[test]
fn profiles_without_subcommand_lists_profiles() {
    let command = parse(os_vec(&["ccs", "profiles"])).unwrap();
    assert_eq!(command, Command::Profiles(ProfilesCommand::List));
}

#[test]
fn profiles_ls_is_list_alias() {
    let command = parse(os_vec(&["ccs", "profiles", "ls"])).unwrap();
    assert_eq!(command, Command::Profiles(ProfilesCommand::List));
}
```

- [ ] **Step 2: Run the parser tests and get the expected failure**

Run: `cargo test --test cli`

Expected: build fails because the Rust package and `ccs::cli` do not exist.

- [ ] **Step 3: Create the Rust package and parser**

Create `Cargo.toml`:

```toml
[package]
name = "ccs"
version = "0.1.0"
edition = "2024"
license = "MIT"
repository = "https://github.com/reedchan7/ccs"
description = "Claude Code session switcher"

[dependencies]
anyhow = "1"
clap = { version = "4", features = ["derive"] }
directories = "6"
dotenvy = "0.15"
serde_json = "1"

[dev-dependencies]
assert_cmd = "2"
predicates = "3"
tempfile = "3"
```

Create `src/lib.rs`:

```rust
pub mod cli;
```

Create `src/main.rs`:

```rust
use anyhow::Result;

fn main() {
    if let Err(error) = run() {
        eprintln!("{error:#}");
        std::process::exit(1);
    }
}

fn run() -> Result<()> {
    let command = ccs::cli::parse(std::env::args_os())?;
    println!("{command:?}");
    Ok(())
}
```

Create `src/cli.rs`:

```rust
use std::ffi::OsString;

use anyhow::{bail, Result};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Command {
    LaunchDefault,
    LaunchAgent {
        agent: String,
        claude_args: Vec<OsString>,
    },
    Use {
        agent: String,
        global: bool,
    },
    Profiles(ProfilesCommand),
    Init {
        hooks_only: bool,
    },
    Status,
    Update,
    PermissionsBypass,
    InternalEnv {
        agent: String,
    },
    Help,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ProfilesCommand {
    List,
    Edit { agent: String },
    Add { agent: String },
    Remove { agent: String, yes: bool },
}

pub fn parse<I, S>(args: I) -> Result<Command>
where
    I: IntoIterator<Item = S>,
    S: Into<OsString>,
{
    let mut args: Vec<OsString> = args.into_iter().map(Into::into).collect();
    if !args.is_empty() {
        args.remove(0);
    }

    let Some(first) = args.first().and_then(|value| value.to_str()) else {
        return Ok(Command::LaunchDefault);
    };

    match first {
        "-h" | "--help" | "help" => Ok(Command::Help),
        "use" => parse_use(&args[1..]),
        "profiles" => parse_profiles(&args[1..]),
        "init" => Ok(Command::Init {
            hooks_only: args.iter().any(|arg| arg == "--hooks-only"),
        }),
        "status" => Ok(Command::Status),
        "update" => Ok(Command::Update),
        "permissions" => parse_permissions(&args[1..]),
        "internal" => parse_internal(&args[1..]),
        _ => Ok(Command::LaunchAgent {
            agent: first.to_owned(),
            claude_args: args[1..].to_vec(),
        }),
    }
}

fn parse_use(args: &[OsString]) -> Result<Command> {
    let Some(agent) = args.first().and_then(|value| value.to_str()) else {
        bail!("usage: ccs use <agent> [--global]");
    };
    let global = args.iter().skip(1).any(|arg| arg == "--global");
    Ok(Command::Use {
        agent: agent.to_owned(),
        global,
    })
}

fn parse_profiles(args: &[OsString]) -> Result<Command> {
    let Some(subcommand) = args.first().and_then(|value| value.to_str()) else {
        return Ok(Command::Profiles(ProfilesCommand::List));
    };

    match subcommand {
        "list" | "ls" => Ok(Command::Profiles(ProfilesCommand::List)),
        "edit" => Ok(Command::Profiles(ProfilesCommand::Edit {
            agent: required_arg(args, "agent")?,
        })),
        "add" => Ok(Command::Profiles(ProfilesCommand::Add {
            agent: required_arg(args, "agent")?,
        })),
        "remove" | "rm" => Ok(Command::Profiles(ProfilesCommand::Remove {
            agent: required_arg(args, "agent")?,
            yes: args.iter().any(|arg| arg == "--yes"),
        })),
        _ => bail!("unknown profiles command: {subcommand}"),
    }
}

fn parse_permissions(args: &[OsString]) -> Result<Command> {
    match args.first().and_then(|value| value.to_str()) {
        Some("bypass") => Ok(Command::PermissionsBypass),
        Some(other) => bail!("unknown permissions command: {other}"),
        None => bail!("usage: ccs permissions bypass"),
    }
}

fn parse_internal(args: &[OsString]) -> Result<Command> {
    match args.first().and_then(|value| value.to_str()) {
        Some("env") => Ok(Command::InternalEnv {
            agent: required_arg(args, "agent")?,
        }),
        Some(other) => bail!("unknown internal command: {other}"),
        None => bail!("usage: ccs internal env <agent>"),
    }
}

fn required_arg(args: &[OsString], name: &str) -> Result<String> {
    args.get(1)
        .and_then(|value| value.to_str())
        .map(ToOwned::to_owned)
        .ok_or_else(|| anyhow::anyhow!("missing {name}"))
}
```

- [ ] **Step 4: Run the parser tests**

Run: `cargo test --test cli`

Expected: all tests in `tests/cli.rs` pass.

- [ ] **Step 5: Commit**

```bash
git add Cargo.toml Cargo.lock src/lib.rs src/main.rs src/cli.rs tests/cli.rs
git commit -m "feat: scaffold rust cli parser"
```

## Task 2: Agent Canonicalization and Built-In Profile Templates

**Files:**
- Create: `src/agent.rs`
- Modify: `src/lib.rs`
- Create: `tests/agents.rs`

**Interfaces:**
- Consumes: `Command` agent strings from `src/cli.rs`.
- Produces: `ccs::agent::Agent::parse(value: &str) -> anyhow::Result<Agent>`.
- Produces: `Agent::canonical(&self) -> &'static str`.
- Produces: `Agent::template(&self, home: &std::path::Path) -> Vec<(String, String)>`.

- [ ] **Step 1: Add alias and template tests**

Create `tests/agents.rs`:

```rust
use ccs::agent::Agent;

#[test]
fn ds_is_deepseek_alias() {
    let agent = Agent::parse("ds").unwrap();
    assert_eq!(agent.canonical(), "deepseek");
}

#[test]
fn deepseek_template_contains_required_model_env() {
    let home = std::path::Path::new("/tmp/home");
    let profile = Agent::parse("deepseek").unwrap().template(home);
    let keys: Vec<_> = profile.iter().map(|(key, _)| key.as_str()).collect();
    assert!(keys.contains(&"ANTHROPIC_BASE_URL"));
    assert!(keys.contains(&"ANTHROPIC_AUTH_TOKEN"));
    assert!(keys.contains(&"ANTHROPIC_DEFAULT_HAIKU_MODEL"));
    assert!(keys.contains(&"CLAUDE_CODE_SUBAGENT_MODEL"));
}

#[test]
fn unknown_agent_is_rejected() {
    let error = Agent::parse("random").unwrap_err().to_string();
    assert!(error.contains("unknown agent"));
}
```

- [ ] **Step 2: Run the tests and get the expected failure**

Run: `cargo test --test agents`

Expected: build fails because `ccs::agent` does not exist.

- [ ] **Step 3: Implement agent parsing and templates**

Modify `src/lib.rs`:

```rust
pub mod agent;
pub mod cli;
```

Create `src/agent.rs`:

```rust
use std::path::Path;

use anyhow::{bail, Result};

const SHARED_PATHS: &str = "CLAUDE.md,settings.json,skills,plugins,rules";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Agent {
    Max,
    Api,
    Glm,
    Mimo,
    Deepseek,
    Kimi,
}

impl Agent {
    pub fn parse(value: &str) -> Result<Self> {
        match value {
            "max" => Ok(Self::Max),
            "api" => Ok(Self::Api),
            "glm" => Ok(Self::Glm),
            "mimo" => Ok(Self::Mimo),
            "deepseek" | "ds" => Ok(Self::Deepseek),
            "kimi" => Ok(Self::Kimi),
            other => bail!("unknown agent '{other}'. expected max, api, glm, mimo, deepseek, ds, or kimi"),
        }
    }

    pub fn canonical(self) -> &'static str {
        match self {
            Self::Max => "max",
            Self::Api => "api",
            Self::Glm => "glm",
            Self::Mimo => "mimo",
            Self::Deepseek => "deepseek",
            Self::Kimi => "kimi",
        }
    }

    pub fn all() -> &'static [Self] {
        &[Self::Max, Self::Api, Self::Glm, Self::Mimo, Self::Deepseek, Self::Kimi]
    }

    pub fn template(self, home: &Path) -> Vec<(String, String)> {
        let config_dir = home
            .join(".config")
            .join("ccs")
            .join("claude")
            .join(self.canonical());
        let mut values = vec![
            ("CLAUDE_CONFIG_DIR".into(), config_dir.display().to_string()),
            ("CCS_SHARED_CLAUDE_DIR".into(), home.join(".claude").display().to_string()),
            ("CCS_SHARED_PATHS".into(), SHARED_PATHS.into()),
        ];

        match self {
            Self::Max => {}
            Self::Api => {
                values.push(("ANTHROPIC_API_KEY".into(), String::new()));
            }
            Self::Glm => {
                values.extend([
                    ("ANTHROPIC_BASE_URL".into(), "https://api.z.ai/api/anthropic".into()),
                    ("ANTHROPIC_AUTH_TOKEN".into(), String::new()),
                    ("ANTHROPIC_DEFAULT_OPUS_MODEL".into(), "glm-5.2[1m]".into()),
                    ("ANTHROPIC_DEFAULT_SONNET_MODEL".into(), "glm-5.2[1m]".into()),
                    ("ANTHROPIC_DEFAULT_HAIKU_MODEL".into(), "glm-4.7".into()),
                    ("API_TIMEOUT_MS".into(), "3000000".into()),
                    ("CLAUDE_CODE_AUTO_COMPACT_WINDOW".into(), "1000000".into()),
                    ("CLAUDE_CODE_DISABLE_NONESSENTIAL_TRAFFIC".into(), "1".into()),
                ]);
            }
            Self::Mimo => {
                values.extend([
                    ("ANTHROPIC_BASE_URL".into(), "https://api.xiaomimimo.com/anthropic".into()),
                    ("ANTHROPIC_AUTH_TOKEN".into(), String::new()),
                    ("ANTHROPIC_DEFAULT_OPUS_MODEL".into(), "mimo-v2.5-pro".into()),
                    ("ANTHROPIC_DEFAULT_SONNET_MODEL".into(), "mimo-v2.5".into()),
                    ("ANTHROPIC_DEFAULT_HAIKU_MODEL".into(), "mimo-v2.5".into()),
                ]);
            }
            Self::Deepseek => {
                values.extend([
                    ("ANTHROPIC_BASE_URL".into(), "https://api.deepseek.com/anthropic".into()),
                    ("ANTHROPIC_AUTH_TOKEN".into(), String::new()),
                    ("ANTHROPIC_DEFAULT_OPUS_MODEL".into(), "deepseek-v4-pro".into()),
                    ("ANTHROPIC_DEFAULT_SONNET_MODEL".into(), "deepseek-v4-pro".into()),
                    ("ANTHROPIC_DEFAULT_HAIKU_MODEL".into(), "deepseek-v4-flash".into()),
                    ("CLAUDE_CODE_SUBAGENT_MODEL".into(), "deepseek-v4-flash".into()),
                    ("CLAUDE_CODE_EFFORT_LEVEL".into(), "max".into()),
                ]);
            }
            Self::Kimi => {
                values.extend([
                    ("ANTHROPIC_BASE_URL".into(), "https://api.kimi.com/coding/".into()),
                    ("ANTHROPIC_AUTH_TOKEN".into(), String::new()),
                    ("ANTHROPIC_DEFAULT_OPUS_MODEL".into(), "kimi-for-coding".into()),
                    ("ANTHROPIC_DEFAULT_SONNET_MODEL".into(), "kimi-for-coding".into()),
                    ("ANTHROPIC_DEFAULT_HAIKU_MODEL".into(), "kimi-for-coding".into()),
                    ("CLAUDE_CODE_SUBAGENT_MODEL".into(), "kimi-for-coding".into()),
                ]);
            }
        }

        values
    }
}
```

- [ ] **Step 4: Run the agent tests**

Run: `cargo test --test agents`

Expected: all tests in `tests/agents.rs` pass.

- [ ] **Step 5: Commit**

```bash
git add src/lib.rs src/agent.rs tests/agents.rs
git commit -m "feat: add built-in agent profiles"
```

## Task 3: Paths, Profile Files, Defaults, and Env Rendering

**Files:**
- Create: `src/paths.rs`
- Create: `src/profile.rs`
- Create: `src/env.rs`
- Modify: `src/lib.rs`
- Create: `tests/profiles.rs`

**Interfaces:**
- Consumes: `Agent` from `src/agent.rs`.
- Produces: `Paths::from_home(home: impl AsRef<Path>) -> Paths`.
- Produces: `Profile::load(paths: &Paths, agent: Agent) -> anyhow::Result<Profile>`.
- Produces: `Profile::write_template(paths: &Paths, agent: Agent) -> anyhow::Result<PathBuf>`.
- Produces: `write_default_profile(paths: &Paths, agent: Agent) -> anyhow::Result<()>`.
- Produces: `read_default_profile(paths: &Paths) -> anyhow::Result<Option<Agent>>`.
- Produces: `render_shell_exports(profile: &Profile, agent: Agent) -> String`.

- [ ] **Step 1: Add profile and env tests**

Create `tests/profiles.rs`:

```rust
use ccs::agent::Agent;
use ccs::env::render_shell_exports;
use ccs::paths::Paths;
use ccs::profile::{read_default_profile, write_default_profile, Profile};
use tempfile::TempDir;

#[test]
fn writes_and_reads_canonical_default_profile() {
    let home = TempDir::new().unwrap();
    let paths = Paths::from_home(home.path());
    write_default_profile(&paths, Agent::Deepseek).unwrap();
    assert_eq!(read_default_profile(&paths).unwrap(), Some(Agent::Deepseek));
    let config = std::fs::read_to_string(paths.config_file()).unwrap();
    assert_eq!(config, "default_profile=deepseek\n");
}

#[test]
fn loads_existing_dotenv_profile() {
    let home = TempDir::new().unwrap();
    let paths = Paths::from_home(home.path());
    std::fs::create_dir_all(paths.profiles_dir()).unwrap();
    std::fs::write(
        paths.profile_file(Agent::Api),
        "CLAUDE_CONFIG_DIR=/tmp/api\nANTHROPIC_API_KEY=secret\n",
    )
    .unwrap();

    let profile = Profile::load(&paths, Agent::Api).unwrap();
    assert_eq!(profile.value("CLAUDE_CONFIG_DIR"), Some("/tmp/api"));
    assert_eq!(profile.value("ANTHROPIC_API_KEY"), Some("secret"));
}

#[test]
fn shell_exports_clear_known_vars_and_hide_ccs_internal_keys() {
    let home = TempDir::new().unwrap();
    let paths = Paths::from_home(home.path());
    std::fs::create_dir_all(paths.profiles_dir()).unwrap();
    std::fs::write(
        paths.profile_file(Agent::Api),
        "CLAUDE_CONFIG_DIR=/tmp/api\nCCS_SHARED_PATHS=skills\nANTHROPIC_API_KEY=secret\n",
    )
    .unwrap();

    let profile = Profile::load(&paths, Agent::Api).unwrap();
    let exports = render_shell_exports(&profile, Agent::Api);
    assert!(exports.contains("unset ANTHROPIC_AUTH_TOKEN"));
    assert!(exports.contains("export CLAUDE_CONFIG_DIR="));
    assert!(exports.contains("export ANTHROPIC_API_KEY="));
    assert!(exports.contains("export CCS_ACTIVE_PROFILE="));
    assert!(!exports.contains("export CCS_SHARED_PATHS="));
}
```

- [ ] **Step 2: Run the profile tests and get the expected failure**

Run: `cargo test --test profiles`

Expected: build fails because the paths, profile, and env modules do not exist.

- [ ] **Step 3: Implement paths**

Modify `src/lib.rs`:

```rust
pub mod agent;
pub mod cli;
pub mod env;
pub mod paths;
pub mod profile;
```

Create `src/paths.rs`:

```rust
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
        self.profiles_dir().join(format!("{}.env", agent.canonical()))
    }

    pub fn claude_dir(&self, agent: Agent) -> PathBuf {
        self.ccs_home.join("claude").join(agent.canonical())
    }

    pub fn hook_file(&self) -> PathBuf {
        self.ccs_home.join("ccs.sh")
    }
}
```

- [ ] **Step 4: Implement profile and default config**

Create `src/profile.rs`:

```rust
use std::collections::BTreeMap;
use std::fs::{self, File};
use std::io::Write;
use std::os::unix::fs::PermissionsExt;
use std::path::PathBuf;

use anyhow::{bail, Context, Result};

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
            let (key, value) = item.with_context(|| format!("invalid profile file {}", file.display()))?;
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
```

- [ ] **Step 5: Implement env rendering**

Create `src/env.rs`:

```rust
use crate::agent::Agent;
use crate::profile::Profile;

pub const KNOWN_ENV_VARS: &[&str] = &[
    "CLAUDE_CONFIG_DIR",
    "CCS_SHARED_CLAUDE_DIR",
    "CCS_SHARED_PATHS",
    "ANTHROPIC_API_KEY",
    "ANTHROPIC_AUTH_TOKEN",
    "ANTHROPIC_BASE_URL",
    "ANTHROPIC_MODEL",
    "ANTHROPIC_DEFAULT_OPUS_MODEL",
    "ANTHROPIC_DEFAULT_SONNET_MODEL",
    "ANTHROPIC_DEFAULT_HAIKU_MODEL",
    "ENABLE_TOOL_SEARCH",
    "CLAUDE_CODE_DISABLE_EXPERIMENTAL_BETAS",
    "CLAUDE_CODE_SUBAGENT_MODEL",
    "CLAUDE_CODE_EFFORT_LEVEL",
    "API_TIMEOUT_MS",
    "CLAUDE_CODE_AUTO_COMPACT_WINDOW",
    "CLAUDE_CODE_DISABLE_NONESSENTIAL_TRAFFIC",
    "CCS_ACTIVE_PROFILE",
];

pub fn render_shell_exports(profile: &Profile, agent: Agent) -> String {
    let mut output = String::new();
    for key in KNOWN_ENV_VARS {
        output.push_str(&format!("unset {key}\n"));
    }
    for (key, value) in profile.iter() {
        if key.starts_with("CCS_") {
            continue;
        }
        output.push_str(&format!("export {key}={}\n", shell_quote(value)));
    }
    output.push_str(&format!(
        "export CCS_ACTIVE_PROFILE={}\n",
        shell_quote(agent.canonical())
    ));
    output
}

fn shell_quote(value: &str) -> String {
    format!("'{}'", value.replace('\'', "'\\''"))
}
```

- [ ] **Step 6: Run the profile tests**

Run: `cargo test --test profiles`

Expected: all tests in `tests/profiles.rs` pass.

- [ ] **Step 7: Commit**

```bash
git add src/lib.rs src/paths.rs src/profile.rs src/env.rs tests/profiles.rs
git commit -m "feat: load profiles and render agent environments"
```

## Task 4: Shared Claude Config Symlinks

**Files:**
- Create: `src/links.rs`
- Modify: `src/lib.rs`
- Modify: `tests/profiles.rs`

**Interfaces:**
- Consumes: `Profile` and `Agent`.
- Produces: `ensure_shared_links(profile: &Profile) -> anyhow::Result<()>`.

- [ ] **Step 1: Add shared link tests**

Append to `tests/profiles.rs`:

```rust
use ccs::links::ensure_shared_links;

#[test]
fn creates_default_shared_symlinks() {
    let home = TempDir::new().unwrap();
    let shared = home.path().join(".claude");
    std::fs::create_dir_all(shared.join("skills")).unwrap();
    std::fs::write(shared.join("settings.json"), "{}\n").unwrap();
    std::fs::write(shared.join("CLAUDE.md"), "base\n").unwrap();

    let config_dir = home.path().join(".config/ccs/claude/max");
    let paths = Paths::from_home(home.path());
    std::fs::create_dir_all(paths.profiles_dir()).unwrap();
    std::fs::write(
        paths.profile_file(Agent::Max),
        format!(
            "CLAUDE_CONFIG_DIR={}\nCCS_SHARED_CLAUDE_DIR={}\nCCS_SHARED_PATHS=CLAUDE.md,settings.json,skills\n",
            config_dir.display(),
            shared.display()
        ),
    )
    .unwrap();

    let profile = Profile::load(&paths, Agent::Max).unwrap();
    ensure_shared_links(&profile).unwrap();

    assert_eq!(std::fs::read_link(config_dir.join("CLAUDE.md")).unwrap(), shared.join("CLAUDE.md"));
    assert_eq!(std::fs::read_link(config_dir.join("settings.json")).unwrap(), shared.join("settings.json"));
    assert_eq!(std::fs::read_link(config_dir.join("skills")).unwrap(), shared.join("skills"));
}

#[test]
fn backs_up_existing_local_shared_path_before_linking() {
    let home = TempDir::new().unwrap();
    let shared = home.path().join(".claude");
    std::fs::create_dir_all(&shared).unwrap();
    std::fs::write(shared.join("settings.json"), "shared\n").unwrap();
    let config_dir = home.path().join(".config/ccs/claude/glm");
    std::fs::create_dir_all(&config_dir).unwrap();
    std::fs::write(config_dir.join("settings.json"), "local\n").unwrap();

    let paths = Paths::from_home(home.path());
    std::fs::create_dir_all(paths.profiles_dir()).unwrap();
    std::fs::write(
        paths.profile_file(Agent::Glm),
        format!(
            "CLAUDE_CONFIG_DIR={}\nCCS_SHARED_CLAUDE_DIR={}\nCCS_SHARED_PATHS=settings.json\nANTHROPIC_BASE_URL=https://example.test\nANTHROPIC_AUTH_TOKEN=token\n",
            config_dir.display(),
            shared.display()
        ),
    )
    .unwrap();

    let profile = Profile::load(&paths, Agent::Glm).unwrap();
    ensure_shared_links(&profile).unwrap();

    assert_eq!(std::fs::read_link(config_dir.join("settings.json")).unwrap(), shared.join("settings.json"));
    assert_eq!(
        std::fs::read_to_string(config_dir.join(".ccs-local-backup/settings.json")).unwrap(),
        "local\n"
    );
}
```

- [ ] **Step 2: Run the profile tests and get the expected failure**

Run: `cargo test --test profiles`

Expected: build fails because `ccs::links` does not exist.

- [ ] **Step 3: Implement symlink handling**

Modify `src/lib.rs`:

```rust
pub mod agent;
pub mod cli;
pub mod env;
pub mod links;
pub mod paths;
pub mod profile;
```

Create `src/links.rs`:

```rust
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
    let shared_paths = profile.value("CCS_SHARED_PATHS").unwrap_or(DEFAULT_SHARED_PATHS);
    for name in shared_paths.split(',').map(str::trim).filter(|value| !value.is_empty()) {
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
```

- [ ] **Step 4: Run the profile tests**

Run: `cargo test --test profiles`

Expected: all tests in `tests/profiles.rs` pass.

- [ ] **Step 5: Commit**

```bash
git add src/lib.rs src/links.rs tests/profiles.rs
git commit -m "feat: share claude config across agents"
```

## Task 5: Direct Agent Launch for `ccs` and `ccs <agent>`

**Files:**
- Create: `src/run.rs`
- Modify: `src/lib.rs`
- Modify: `src/main.rs`
- Create: `tests/support/mod.rs`
- Create: `tests/launch.rs`

**Interfaces:**
- Consumes: `Command`, `Paths`, `Agent`, `Profile`, `render_shell_exports`, and `ensure_shared_links`.
- Produces: `run::execute(command: Command) -> anyhow::Result<i32>`.
- Produces: `run::launch_agent(paths: &Paths, agent: Agent, args: &[OsString]) -> anyhow::Result<i32>`.

- [ ] **Step 1: Add fake-Claude support tests**

Create `tests/support/mod.rs`:

```rust
use std::path::{Path, PathBuf};

use tempfile::TempDir;

pub struct TestHome {
    temp: TempDir,
    bin: PathBuf,
}

impl TestHome {
    pub fn new() -> Self {
        let temp = TempDir::new().unwrap();
        let bin = temp.path().join("bin");
        std::fs::create_dir_all(&bin).unwrap();
        Self { temp, bin }
    }

    pub fn path(&self) -> &Path {
        self.temp.path()
    }

    pub fn bin(&self) -> &Path {
        &self.bin
    }

    pub fn write_fake_claude(&self) {
        let path = self.bin.join("claude");
        std::fs::write(
            &path,
            "#!/usr/bin/env bash\nprintf 'CCS_ACTIVE_PROFILE=%s\\n' \"${CCS_ACTIVE_PROFILE:-}\"\nprintf 'CLAUDE_CONFIG_DIR=%s\\n' \"${CLAUDE_CONFIG_DIR:-}\"\nprintf 'ANTHROPIC_AUTH_TOKEN=%s\\n' \"${ANTHROPIC_AUTH_TOKEN:-}\"\nprintf 'ARGS='\nprintf '%s ' \"$@\"\nprintf '\\n'\n",
        )
        .unwrap();
        use std::os::unix::fs::PermissionsExt;
        let mut permissions = std::fs::metadata(&path).unwrap().permissions();
        permissions.set_mode(0o755);
        std::fs::set_permissions(path, permissions).unwrap();
    }
}
```

Create `tests/launch.rs`:

```rust
mod support;

use assert_cmd::Command;
use predicates::prelude::*;
use support::TestHome;

fn ccs(home: &TestHome) -> Command {
    let mut cmd = Command::cargo_bin("ccs").unwrap();
    let path = format!("{}:{}", home.bin().display(), std::env::var("PATH").unwrap());
    cmd.env("CCS_TEST_HOME", home.path())
        .env("CCS_TEST_NO_EXEC", "1")
        .env("PATH", path);
    cmd
}

#[test]
fn bare_ccs_without_default_prints_next_step() {
    let home = TestHome::new();
    ccs(&home)
        .assert()
        .failure()
        .stderr(predicate::str::contains("No default agent set"));
}

#[test]
fn bare_ccs_runs_default_agent() {
    let home = TestHome::new();
    home.write_fake_claude();
    std::fs::create_dir_all(home.path().join(".config/ccs/profiles")).unwrap();
    std::fs::write(
        home.path().join(".config/ccs/profiles/deepseek.env"),
        format!(
            "CLAUDE_CONFIG_DIR={}\nANTHROPIC_BASE_URL=https://api.deepseek.com/anthropic\nANTHROPIC_AUTH_TOKEN=token\n",
            home.path().join(".config/ccs/claude/deepseek").display()
        ),
    )
    .unwrap();
    std::fs::create_dir_all(home.path().join(".config/ccs")).unwrap();
    std::fs::write(home.path().join(".config/ccs/config"), "default_profile=deepseek\n").unwrap();

    ccs(&home)
        .assert()
        .success()
        .stdout(predicate::str::contains("CCS_ACTIVE_PROFILE=deepseek"));
}

#[test]
fn ccs_ds_runs_deepseek_and_passes_args() {
    let home = TestHome::new();
    home.write_fake_claude();
    std::fs::create_dir_all(home.path().join(".config/ccs/profiles")).unwrap();
    std::fs::write(
        home.path().join(".config/ccs/profiles/deepseek.env"),
        format!(
            "CLAUDE_CONFIG_DIR={}\nANTHROPIC_BASE_URL=https://api.deepseek.com/anthropic\nANTHROPIC_AUTH_TOKEN=token\n",
            home.path().join(".config/ccs/claude/deepseek").display()
        ),
    )
    .unwrap();

    ccs(&home)
        .args(["ds", "--print", "hello"])
        .assert()
        .success()
        .stdout(predicate::str::contains("CCS_ACTIVE_PROFILE=deepseek"))
        .stdout(predicate::str::contains("ARGS=--print hello "));
}
```

- [ ] **Step 2: Run the launch tests and get the expected failure**

Run: `cargo test --test launch`

Expected: tests fail because `main.rs` still prints parsed commands and does not launch Claude.

- [ ] **Step 3: Implement command execution and launch**

Modify `src/lib.rs`:

```rust
pub mod agent;
pub mod cli;
pub mod env;
pub mod links;
pub mod paths;
pub mod profile;
pub mod run;
```

Create `src/run.rs`:

```rust
use std::ffi::OsString;
use std::process::Command as ProcessCommand;

use anyhow::{bail, Result};

use crate::agent::Agent;
use crate::cli::{Command, ProfilesCommand};
use crate::env::KNOWN_ENV_VARS;
use crate::links::ensure_shared_links;
use crate::paths::Paths;
use crate::profile::{read_default_profile, Profile};

pub fn execute(command: Command) -> Result<i32> {
    let paths = Paths::from_env()?;
    match command {
        Command::LaunchDefault => {
            let Some(agent) = read_default_profile(&paths)? else {
                bail!("No default agent set.\nRun: ccs use ds --global\nSee: ccs profiles");
            };
            launch_agent(&paths, agent, &[])
        }
        Command::LaunchAgent { agent, claude_args } => {
            let agent = Agent::parse(&agent)?;
            launch_agent(&paths, agent, &claude_args)
        }
        Command::Profiles(ProfilesCommand::List) => {
            for agent in Agent::all() {
                if paths.profile_file(*agent).exists() {
                    println!("{}", agent.canonical());
                }
            }
            Ok(0)
        }
        Command::Help => {
            print_help();
            Ok(0)
        }
        other => bail!("command not implemented yet: {other:?}"),
    }
}

pub fn launch_agent(paths: &Paths, agent: Agent, args: &[OsString]) -> Result<i32> {
    let profile = Profile::load(paths, agent)?;
    ensure_shared_links(&profile)?;

    let mut command = ProcessCommand::new("claude");
    command.args(args);
    for key in KNOWN_ENV_VARS {
        command.env_remove(key);
    }
    for (key, value) in profile.iter() {
        if !key.starts_with("CCS_") {
            command.env(key, value);
        }
    }
    command.env("CCS_ACTIVE_PROFILE", agent.canonical());

    if std::env::var_os("CCS_TEST_NO_EXEC").is_some() {
        let status = command.status()?;
        return Ok(status.code().unwrap_or(1));
    }

    use std::os::unix::process::CommandExt;
    let error = command.exec();
    Err(error.into())
}

fn print_help() {
    println!("Usage:");
    println!("  ccs [agent] [claude args...]");
    println!("  ccs use <agent> [--global]");
    println!("  ccs profiles [ls|list|edit|add|remove]");
    println!("  ccs init");
    println!("  ccs status");
    println!("  ccs update");
}
```

Modify `src/main.rs`:

```rust
use anyhow::Result;

fn main() {
    match run() {
        Ok(code) => std::process::exit(code),
        Err(error) => {
            eprintln!("{error:#}");
            std::process::exit(1);
        }
    }
}

fn run() -> Result<i32> {
    let command = ccs::cli::parse(std::env::args_os())?;
    ccs::run::execute(command)
}
```

- [ ] **Step 4: Run launch tests**

Run: `cargo test --test launch`

Expected: all tests in `tests/launch.rs` pass.

- [ ] **Step 5: Commit**

```bash
git add src/lib.rs src/main.rs src/run.rs tests/support/mod.rs tests/launch.rs
git commit -m "feat: launch claude agents from ccs"
```

## Task 6: `ccs use`, Global Defaults, Shell Hook, and Init

**Files:**
- Create: `src/shell.rs`
- Modify: `src/lib.rs`
- Modify: `src/run.rs`
- Create: `tests/shell.rs`

**Interfaces:**
- Consumes: `Command::Use`, `Command::InternalEnv`, `Command::Init`.
- Produces: `shell::render_hook(binary_path: &str) -> String`.
- Produces: `shell::install_hooks(paths: &Paths, binary_path: &str) -> anyhow::Result<()>`.
- Produces: public behavior for `ccs use <agent>`, `ccs use <agent> --global`, `ccs internal env use <agent>`, and `ccs init --hooks-only`.

- [ ] **Step 1: Add shell behavior tests**

Create `tests/shell.rs`:

```rust
mod support;

use assert_cmd::Command;
use predicates::prelude::*;
use support::TestHome;

fn ccs(home: &TestHome) -> Command {
    let mut cmd = Command::cargo_bin("ccs").unwrap();
    cmd.env("CCS_TEST_HOME", home.path());
    cmd
}

fn write_deepseek(home: &TestHome) {
    std::fs::create_dir_all(home.path().join(".config/ccs/profiles")).unwrap();
    std::fs::write(
        home.path().join(".config/ccs/profiles/deepseek.env"),
        format!(
            "CLAUDE_CONFIG_DIR={}\nANTHROPIC_BASE_URL=https://api.deepseek.com/anthropic\nANTHROPIC_AUTH_TOKEN=token\n",
            home.path().join(".config/ccs/claude/deepseek").display()
        ),
    )
    .unwrap();
}

#[test]
fn use_global_writes_default_profile() {
    let home = TestHome::new();
    write_deepseek(&home);
    ccs(&home)
        .args(["use", "ds", "--global"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Default agent: deepseek"));
    let config = std::fs::read_to_string(home.path().join(".config/ccs/config")).unwrap();
    assert_eq!(config, "default_profile=deepseek\n");
}

#[test]
fn internal_env_renders_shell_exports() {
    let home = TestHome::new();
    write_deepseek(&home);
    ccs(&home)
        .args(["internal", "env", "use", "ds"])
        .assert()
        .success()
        .stdout(predicate::str::contains("unset ANTHROPIC_API_KEY"))
        .stdout(predicate::str::contains("export CCS_ACTIVE_PROFILE='deepseek'"));
}

#[test]
fn init_hooks_only_writes_hook_file() {
    let home = TestHome::new();
    ccs(&home)
        .args(["init", "--hooks-only"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Shell hook installed"));
    let hook = std::fs::read_to_string(home.path().join(".config/ccs/ccs.sh")).unwrap();
    assert!(hook.contains("ccs()"));
    assert!(hook.contains("internal env use"));
}
```

- [ ] **Step 2: Run shell tests and get the expected failure**

Run: `cargo test --test shell`

Expected: tests fail because `use`, `internal env`, and `init` are not implemented yet.

- [ ] **Step 3: Adjust internal parser for `internal env use <agent>`**

Modify `src/cli.rs` internal parsing:

```rust
fn parse_internal(args: &[OsString]) -> Result<Command> {
    match args.first().and_then(|value| value.to_str()) {
        Some("env") => {
            if args.get(1).and_then(|value| value.to_str()) != Some("use") {
                bail!("usage: ccs internal env use <agent>");
            }
            let agent = args
                .get(2)
                .and_then(|value| value.to_str())
                .ok_or_else(|| anyhow::anyhow!("missing agent"))?;
            Ok(Command::InternalEnv {
                agent: agent.to_owned(),
            })
        }
        Some(other) => bail!("unknown internal command: {other}"),
        None => bail!("usage: ccs internal env use <agent>"),
    }
}
```

- [ ] **Step 4: Implement hook rendering and installation**

Modify `src/lib.rs`:

```rust
pub mod agent;
pub mod cli;
pub mod env;
pub mod links;
pub mod paths;
pub mod profile;
pub mod run;
pub mod shell;
```

Create `src/shell.rs`:

```rust
use std::fs;

use anyhow::Result;

use crate::paths::Paths;

pub fn render_hook(binary_path: &str) -> String {
    format!(
        r#"# >>> ccs shell hook >>>
ccs() {{
  local command="${{1:-}}"
  case " $* " in
    *" --global "*) "{binary_path}" "$@" ;;
    *) if [ "$command" = "use" ]; then
         local output
         output="$("{binary_path}" internal env "$@")" || {{
           printf '%s\n' "$output" >&2
           return 1
         }}
         eval "$output"
       else
         "{binary_path}" "$@"
       fi ;;
  esac
}}
# <<< ccs shell hook <<<
"#
    )
}

pub fn install_hooks(paths: &Paths, binary_path: &str) -> Result<()> {
    fs::create_dir_all(paths.ccs_home())?;
    fs::write(paths.hook_file(), render_hook(binary_path))?;
    append_source_line(&paths.home().join(".zshrc"))?;
    append_source_line(&paths.home().join(".bashrc"))?;
    Ok(())
}

fn append_source_line(rc_file: &std::path::Path) -> Result<()> {
    let line = r#"[ -f "$HOME/.config/ccs/ccs.sh" ] && . "$HOME/.config/ccs/ccs.sh""#;
    let existing = fs::read_to_string(rc_file).unwrap_or_default();
    if existing.contains(line) {
        return Ok(());
    }
    let mut next = existing;
    if !next.ends_with('\n') && !next.is_empty() {
        next.push('\n');
    }
    next.push_str(line);
    next.push('\n');
    fs::write(rc_file, next)?;
    Ok(())
}
```

- [ ] **Step 5: Implement use, internal env, and init execution**

Modify the relevant match arms in `src/run.rs`:

```rust
use crate::env::render_shell_exports;
use crate::profile::{read_default_profile, write_default_profile, Profile};
use crate::shell;
```

Add arms:

```rust
Command::Use { agent, global } => {
    let agent = Agent::parse(&agent)?;
    if global {
        write_default_profile(&paths, agent)?;
        println!("Default agent: {}", agent.canonical());
    } else {
        println!("Run this once, or run `ccs init` to install the shell hook:");
        println!("eval \"$({} internal env use {})\"", current_binary(), agent.canonical());
    }
    Ok(0)
}
Command::InternalEnv { agent } => {
    let agent = Agent::parse(&agent)?;
    let profile = Profile::load(&paths, agent)?;
    ensure_shared_links(&profile)?;
    print!("{}", render_shell_exports(&profile, agent));
    Ok(0)
}
Command::Init { hooks_only } => {
    shell::install_hooks(&paths, &current_binary())?;
    if hooks_only {
        println!("Shell hook installed");
    } else {
        println!("Shell hook installed");
        println!("Next: ccs profiles add ds");
    }
    Ok(0)
}
```

Add helper:

```rust
fn current_binary() -> String {
    std::env::current_exe()
        .ok()
        .map(|path| path.display().to_string())
        .unwrap_or_else(|| "ccs".into())
}
```

- [ ] **Step 6: Run shell tests**

Run: `cargo test --test shell`

Expected: all tests in `tests/shell.rs` pass.

- [ ] **Step 7: Commit**

```bash
git add src/lib.rs src/cli.rs src/run.rs src/shell.rs tests/shell.rs
git commit -m "feat: add shell agent switching"
```

## Task 7: Profile Management and Status Output

**Files:**
- Modify: `src/run.rs`
- Modify: `tests/cli.rs`
- Create: `tests/profile_commands.rs`

**Interfaces:**
- Consumes: `ProfilesCommand` from `src/cli.rs`.
- Produces: public behavior for `ccs profiles`, `ccs profiles ls`, `ccs profiles edit <agent>`, `ccs profiles add <agent>`, `ccs profiles remove <agent> --yes`, and `ccs status`.

- [ ] **Step 1: Add profile command tests**

Create `tests/profile_commands.rs`:

```rust
mod support;

use assert_cmd::Command;
use predicates::prelude::*;
use support::TestHome;

fn ccs(home: &TestHome) -> Command {
    let mut cmd = Command::cargo_bin("ccs").unwrap();
    cmd.env("CCS_TEST_HOME", home.path());
    cmd
}

#[test]
fn profiles_list_does_not_print_secrets() {
    let home = TestHome::new();
    std::fs::create_dir_all(home.path().join(".config/ccs/profiles")).unwrap();
    std::fs::write(
        home.path().join(".config/ccs/profiles/kimi.env"),
        "CLAUDE_CONFIG_DIR=/tmp/kimi\nANTHROPIC_BASE_URL=https://api.kimi.com/coding/\nANTHROPIC_AUTH_TOKEN=secret\n",
    )
    .unwrap();

    ccs(&home)
        .args(["profiles", "ls"])
        .assert()
        .success()
        .stdout(predicate::str::contains("kimi"))
        .stdout(predicate::str::contains("secret").not());
}

#[test]
fn profiles_add_creates_builtin_stub() {
    let home = TestHome::new();
    ccs(&home)
        .args(["profiles", "add", "ds"])
        .assert()
        .success()
        .stdout(predicate::str::contains("deepseek"));
    assert!(home.path().join(".config/ccs/profiles/deepseek.env").exists());
}

#[test]
fn profiles_remove_yes_deletes_profile_file() {
    let home = TestHome::new();
    let file = home.path().join(".config/ccs/profiles/deepseek.env");
    std::fs::create_dir_all(file.parent().unwrap()).unwrap();
    std::fs::write(&file, "CLAUDE_CONFIG_DIR=/tmp/deepseek\n").unwrap();

    ccs(&home)
        .args(["profiles", "remove", "ds", "--yes"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Removed deepseek"));
    assert!(!file.exists());
}

#[test]
fn status_prints_current_default_and_profiles_without_secrets() {
    let home = TestHome::new();
    std::fs::create_dir_all(home.path().join(".config/ccs/profiles")).unwrap();
    std::fs::write(
        home.path().join(".config/ccs/profiles/deepseek.env"),
        "CLAUDE_CONFIG_DIR=/tmp/deepseek\nANTHROPIC_BASE_URL=https://api.deepseek.com/anthropic\nANTHROPIC_AUTH_TOKEN=secret\n",
    )
    .unwrap();
    std::fs::create_dir_all(home.path().join(".config/ccs")).unwrap();
    std::fs::write(home.path().join(".config/ccs/config"), "default_profile=deepseek\n").unwrap();

    ccs(&home)
        .arg("status")
        .assert()
        .success()
        .stdout(predicate::str::contains("Default agent: deepseek"))
        .stdout(predicate::str::contains("deepseek"))
        .stdout(predicate::str::contains("secret").not());
}
```

- [ ] **Step 2: Run profile command tests and get the expected failure**

Run: `cargo test --test profile_commands`

Expected: tests fail because profile management and status are not complete.

- [ ] **Step 3: Implement profile management arms**

Modify `src/run.rs` profile command handling:

```rust
Command::Profiles(ProfilesCommand::List) => {
    for agent in Agent::all() {
        let file = paths.profile_file(*agent);
        if file.exists() {
            println!("{} -> {}", agent.canonical(), file.display());
        }
    }
    Ok(0)
}
Command::Profiles(ProfilesCommand::Add { agent }) => {
    let agent = Agent::parse(&agent)?;
    let file = crate::profile::write_template(&paths, agent)?;
    println!("Added {} profile: {}", agent.canonical(), file.display());
    Ok(0)
}
Command::Profiles(ProfilesCommand::Edit { agent }) => {
    let agent = Agent::parse(&agent)?;
    let file = crate::profile::write_template(&paths, agent)?;
    let editor = std::env::var("EDITOR").unwrap_or_else(|_| "vi".into());
    let status = std::process::Command::new(editor).arg(file).status()?;
    Ok(status.code().unwrap_or(1))
}
Command::Profiles(ProfilesCommand::Remove { agent, yes }) => {
    let agent = Agent::parse(&agent)?;
    if !yes {
        anyhow::bail!("refusing to remove {} without --yes", agent.canonical());
    }
    let file = paths.profile_file(agent);
    if file.exists() {
        std::fs::remove_file(&file)?;
    }
    println!("Removed {}", agent.canonical());
    Ok(0)
}
```

Add status arm:

```rust
Command::Status => {
    let active = std::env::var("CCS_ACTIVE_PROFILE").unwrap_or_else(|_| "none".into());
    let default = read_default_profile(&paths)?
        .map(|agent| agent.canonical().to_owned())
        .unwrap_or_else(|| "none".into());
    println!("Active agent: {active}");
    println!("Default agent: {default}");
    println!("Profiles:");
    for agent in Agent::all() {
        let file = paths.profile_file(*agent);
        if file.exists() {
            println!("  {} -> {}", agent.canonical(), file.display());
        }
    }
    Ok(0)
}
```

- [ ] **Step 4: Run profile command tests**

Run: `cargo test --test profile_commands`

Expected: all tests in `tests/profile_commands.rs` pass.

- [ ] **Step 5: Commit**

```bash
git add src/run.rs tests/profile_commands.rs
git commit -m "feat: manage ccs profiles"
```

## Task 8: Claude Permissions Bypass Command

**Files:**
- Create: `src/permissions.rs`
- Modify: `src/lib.rs`
- Modify: `src/run.rs`
- Create: `tests/permissions.rs`

**Interfaces:**
- Consumes: `Command::PermissionsBypass`.
- Produces: `permissions::set_bypass_permissions(project_dir: &Path) -> anyhow::Result<PathBuf>`.

- [ ] **Step 1: Add permissions tests**

Create `tests/permissions.rs`:

```rust
use ccs::permissions::set_bypass_permissions;
use serde_json::json;
use tempfile::TempDir;

#[test]
fn creates_settings_local_json_with_bypass_permissions() {
    let project = TempDir::new().unwrap();
    let file = set_bypass_permissions(project.path()).unwrap();
    let value: serde_json::Value =
        serde_json::from_str(&std::fs::read_to_string(file).unwrap()).unwrap();
    assert_eq!(value["permissions"]["defaultMode"], "bypassPermissions");
}

#[test]
fn preserves_existing_json_fields() {
    let project = TempDir::new().unwrap();
    std::fs::create_dir_all(project.path().join(".claude")).unwrap();
    std::fs::write(
        project.path().join(".claude/settings.local.json"),
        json!({"someOtherSetting": true, "permissions": {"allow": []}}).to_string(),
    )
    .unwrap();

    let file = set_bypass_permissions(project.path()).unwrap();
    let value: serde_json::Value =
        serde_json::from_str(&std::fs::read_to_string(file).unwrap()).unwrap();
    assert_eq!(value["someOtherSetting"], true);
    assert_eq!(value["permissions"]["allow"], json!([]));
    assert_eq!(value["permissions"]["defaultMode"], "bypassPermissions");
}
```

- [ ] **Step 2: Run permissions tests and get the expected failure**

Run: `cargo test --test permissions`

Expected: build fails because `ccs::permissions` does not exist.

- [ ] **Step 3: Implement permissions JSON mutation**

Modify `src/lib.rs`:

```rust
pub mod agent;
pub mod cli;
pub mod env;
pub mod links;
pub mod paths;
pub mod permissions;
pub mod profile;
pub mod run;
pub mod shell;
```

Create `src/permissions.rs`:

```rust
use std::fs;
use std::path::{Path, PathBuf};

use anyhow::Result;
use serde_json::{json, Value};

pub fn set_bypass_permissions(project_dir: &Path) -> Result<PathBuf> {
    let claude_dir = project_dir.join(".claude");
    fs::create_dir_all(&claude_dir)?;
    let file = claude_dir.join("settings.local.json");
    let mut value: Value = if file.exists() {
        serde_json::from_str(&fs::read_to_string(&file)?)?
    } else {
        json!({})
    };
    if !value.is_object() {
        value = json!({});
    }
    let object = value.as_object_mut().expect("object set above");
    let permissions = object.entry("permissions").or_insert_with(|| json!({}));
    if !permissions.is_object() {
        *permissions = json!({});
    }
    permissions
        .as_object_mut()
        .expect("object set above")
        .insert("defaultMode".into(), json!("bypassPermissions"));
    fs::write(&file, serde_json::to_string_pretty(&value)? + "\n")?;
    Ok(file)
}
```

Add run arm:

```rust
Command::PermissionsBypass => {
    let file = crate::permissions::set_bypass_permissions(&std::env::current_dir()?)?;
    println!("Updated {}", file.display());
    Ok(0)
}
```

- [ ] **Step 4: Run permissions tests**

Run: `cargo test --test permissions`

Expected: all tests in `tests/permissions.rs` pass.

- [ ] **Step 5: Commit**

```bash
git add src/lib.rs src/permissions.rs src/run.rs tests/permissions.rs
git commit -m "feat: configure claude bypass permissions"
```

## Task 9: Self-Update Command

**Files:**
- Create: `src/update.rs`
- Modify: `src/lib.rs`
- Modify: `src/run.rs`
- Create: `tests/update.rs`

**Interfaces:**
- Consumes: `Command::Update`.
- Produces: `update::target_triple() -> &'static str`.
- Produces: `update::asset_name(version: &str, target: &str) -> String`.
- Produces: `update::run_update() -> anyhow::Result<()>`.

- [ ] **Step 1: Add release asset tests**

Create `tests/update.rs`:

```rust
use ccs::update::asset_name;

#[test]
fn builds_linux_amd64_asset_name() {
    assert_eq!(
        asset_name("v1.2.3", "x86_64-unknown-linux-gnu"),
        "ccs-v1.2.3-x86_64-unknown-linux-gnu.tar.gz"
    );
}

#[test]
fn builds_macos_arm64_asset_name() {
    assert_eq!(
        asset_name("v1.2.3", "aarch64-apple-darwin"),
        "ccs-v1.2.3-aarch64-apple-darwin.tar.gz"
    );
}
```

- [ ] **Step 2: Run update tests and get the expected failure**

Run: `cargo test --test update`

Expected: build fails because `ccs::update` does not exist.

- [ ] **Step 3: Add self-update dependency**

Run:

```bash
cargo add self_update
```

Expected: `Cargo.toml` and `Cargo.lock` update with the latest compatible `self_update` release.

- [ ] **Step 4: Implement update helpers**

Modify `src/lib.rs`:

```rust
pub mod agent;
pub mod cli;
pub mod env;
pub mod links;
pub mod paths;
pub mod permissions;
pub mod profile;
pub mod run;
pub mod shell;
pub mod update;
```

Create `src/update.rs`:

```rust
use anyhow::{bail, Result};

pub fn target_triple() -> &'static str {
    match (std::env::consts::OS, std::env::consts::ARCH) {
        ("linux", "x86_64") => "x86_64-unknown-linux-gnu",
        ("linux", "aarch64") => "aarch64-unknown-linux-gnu",
        ("macos", "x86_64") => "x86_64-apple-darwin",
        ("macos", "aarch64") => "aarch64-apple-darwin",
        _ => "unsupported",
    }
}

pub fn asset_name(version: &str, target: &str) -> String {
    format!("ccs-{version}-{target}.tar.gz")
}

pub fn run_update() -> Result<()> {
    let target = target_triple();
    if target == "unsupported" {
        bail!(
            "self-update is not available for {} {}",
            std::env::consts::OS,
            std::env::consts::ARCH
        );
    }
    let status = self_update::backends::github::Update::configure()
        .repo_owner("reedchan7")
        .repo_name("ccs")
        .bin_name("ccs")
        .target(target)
        .show_download_progress(true)
        .current_version(env!("CARGO_PKG_VERSION"))
        .build()?
        .update()?;
    println!("Updated to {}", status.version());
    Ok(())
}
```

Add run arm:

```rust
Command::Update => {
    crate::update::run_update()?;
    Ok(0)
}
```

- [ ] **Step 5: Run update tests**

Run: `cargo test --test update`

Expected: all tests in `tests/update.rs` pass.

- [ ] **Step 6: Commit**

```bash
git add Cargo.toml Cargo.lock src/lib.rs src/update.rs src/run.rs tests/update.rs
git commit -m "feat: add github release self-update"
```

## Task 10: Install Script, Makefile, README, and Bash Shim Replacement

**Files:**
- Modify: `install.sh`
- Modify: `bin/ccs`
- Create: `Makefile`
- Modify: `README.md`

**Interfaces:**
- Consumes: compiled Rust binary at `target/release/ccs`.
- Produces: `make fmt`, `make lint`, `make test`, `make build`, `make install`, `make release-local`.
- Produces: install behavior that copies the Rust binary to `~/.local/bin/ccs`.

- [ ] **Step 1: Add Makefile**

Create `Makefile`:

```makefile
.PHONY: fmt lint test build install release-local

fmt:
	cargo fmt --all

lint:
	cargo clippy --all-targets --all-features -- -D warnings

test:
	cargo test --all

build:
	cargo build --locked

install:
	cargo build --release --locked
	mkdir -p "$$HOME/.local/bin"
	cp target/release/ccs "$$HOME/.local/bin/ccs"
	"$$HOME/.local/bin/ccs" init --hooks-only

release-local:
	cargo build --release --locked
```

- [ ] **Step 2: Replace install script with Rust binary installer**

Modify `install.sh`:

```bash
#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
INSTALL_BIN_DIR="${HOME}/.local/bin"
INSTALL_BIN_PATH="${INSTALL_BIN_DIR}/ccs"

main() {
  cargo build --release --locked --manifest-path "${ROOT_DIR}/Cargo.toml"
  mkdir -p "${INSTALL_BIN_DIR}"
  cp "${ROOT_DIR}/target/release/ccs" "${INSTALL_BIN_PATH}"
  chmod +x "${INSTALL_BIN_PATH}"
  "${INSTALL_BIN_PATH}" init --hooks-only
  printf 'Installed ccs to %s\n' "${INSTALL_BIN_PATH}"
}

main "$@"
```

- [ ] **Step 3: Replace `bin/ccs` with a dev shim**

Modify `bin/ccs`:

```bash
#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
exec cargo run --quiet --manifest-path "${ROOT_DIR}/Cargo.toml" -- "$@"
```

- [ ] **Step 4: Update README daily workflow**

Replace old quick start and command sections in `README.md` with:

````markdown
## Quick Start

```bash
ccs init
ccs profiles add ds
ccs use ds --global
ccs
```

Daily commands:

```bash
ccs                    # open Claude Code with the default agent
ccs ds                 # open Claude Code with DeepSeek
ccs kimi --print hello # pass args through to Claude Code
ccs use ds             # use DeepSeek for plain `claude` in this shell
ccs use ds --global    # use DeepSeek by default for new shells and bare `ccs`
ccs profiles ls
ccs profiles edit ds
ccs update
```
````

- [ ] **Step 5: Run repository commands**

Run:

```bash
make fmt
make lint
make test
make build
```

Expected: all four targets exit successfully.

- [ ] **Step 6: Commit**

```bash
git add install.sh bin/ccs Makefile README.md
git commit -m "docs: document rust ccs workflow"
```

## Task 11: CI and Tag Release Workflows

**Files:**
- Create: `.github/workflows/ci.yml`
- Create: `.github/workflows/release.yml`

**Interfaces:**
- Produces: CI on push and pull request.
- Produces: release assets on `v*` tags for Linux/macOS AMD64/ARM64.

- [ ] **Step 1: Add CI workflow**

Create `.github/workflows/ci.yml`:

```yaml
name: CI

on:
  pull_request:
  push:
    branches:
      - main

jobs:
  rust:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
        with:
          components: rustfmt, clippy
      - uses: Swatinem/rust-cache@v2
      - run: cargo fmt --all -- --check
      - run: cargo clippy --all-targets --all-features -- -D warnings
      - run: cargo test --all
      - run: cargo build --locked
```

- [ ] **Step 2: Add release workflow**

Create `.github/workflows/release.yml`:

```yaml
name: Release

on:
  push:
    tags:
      - "v*"

permissions:
  contents: write

jobs:
  build:
    strategy:
      fail-fast: false
      matrix:
        include:
          - os: ubuntu-latest
            target: x86_64-unknown-linux-gnu
          - os: ubuntu-24.04-arm
            target: aarch64-unknown-linux-gnu
          - os: macos-13
            target: x86_64-apple-darwin
          - os: macos-latest
            target: aarch64-apple-darwin
    runs-on: ${{ matrix.os }}
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
        with:
          targets: ${{ matrix.target }}
      - uses: Swatinem/rust-cache@v2
      - run: cargo build --release --locked --target ${{ matrix.target }}
      - name: Package
        shell: bash
        run: |
          version="${GITHUB_REF_NAME}"
          asset="ccs-${version}-${{ matrix.target }}"
          mkdir -p "dist/${asset}"
          cp "target/${{ matrix.target }}/release/ccs" "dist/${asset}/ccs"
          cp README.md "dist/${asset}/README.md"
          tar -C dist -czf "dist/${asset}.tar.gz" "${asset}"
          shasum -a 256 "dist/${asset}.tar.gz" > "dist/${asset}.tar.gz.sha256"
      - uses: actions/upload-artifact@v4
        with:
          name: ccs-${{ matrix.target }}
          path: |
            dist/*.tar.gz
            dist/*.sha256

  release:
    needs: build
    runs-on: ubuntu-latest
    steps:
      - uses: actions/download-artifact@v4
        with:
          path: dist
          merge-multiple: true
      - uses: softprops/action-gh-release@v2
        with:
          files: |
            dist/*.tar.gz
            dist/*.sha256
```

- [ ] **Step 3: Run local workflow-equivalent commands**

Run:

```bash
cargo fmt --all -- --check
cargo clippy --all-targets --all-features -- -D warnings
cargo test --all
cargo build --locked
```

Expected: all commands exit successfully.

- [ ] **Step 4: Commit**

```bash
git add .github/workflows/ci.yml .github/workflows/release.yml
git commit -m "ci: build and release ccs binaries"
```

## Task 12: Final Repository Pass

**Files:**
- Modify as needed: files touched by Tasks 1-11 only.

**Interfaces:**
- Consumes: complete Rust rewrite.
- Produces: clean working implementation ready for user acceptance.

- [ ] **Step 1: Run full local suite**

Run:

```bash
make fmt
make lint
make test
make build
```

Expected: all targets exit successfully.

- [ ] **Step 2: Run smoke commands against fake home**

Run:

```bash
tmp_home="$(mktemp -d)"
CCS_TEST_HOME="$tmp_home" target/debug/ccs profiles add ds
CCS_TEST_HOME="$tmp_home" target/debug/ccs use ds --global
CCS_TEST_HOME="$tmp_home" target/debug/ccs profiles ls
CCS_TEST_HOME="$tmp_home" target/debug/ccs status
```

Expected:

```text
deepseek
Default agent: deepseek
deepseek
Default agent: deepseek
```

- [ ] **Step 3: Review git diff for unrelated user edits**

Run:

```bash
git status --short
git diff --stat
```

Expected: only planned Rust rewrite, docs, installer, Makefile, README, and workflow files are changed.

- [ ] **Step 4: Commit any final polish**

If Task 12 required edits:

```bash
git add .
git commit -m "chore: polish rust ccs rewrite"
```

If Task 12 required no edits, do not create an empty commit.
