use std::ffi::OsString;
use std::process::Command as ProcessCommand;

use anyhow::{Result, bail};

use crate::agent::Agent;
use crate::cli::{Command, ProfilesCommand};
use crate::env::{KNOWN_ENV_VARS, render_shell_exports};
use crate::links::ensure_shared_links;
use crate::paths::Paths;
use crate::profile::{Profile, read_default_profile, write_default_profile, write_template};
use crate::shell;

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
                let file = paths.profile_file(*agent);
                if file.exists() {
                    println!("{} -> {}", agent.canonical(), file.display());
                }
            }
            Ok(0)
        }
        Command::Profiles(ProfilesCommand::Add { agent }) => {
            let agent = Agent::parse(&agent)?;
            let file = write_template(&paths, agent)?;
            println!("Added {} profile: {}", agent.canonical(), file.display());
            Ok(0)
        }
        Command::Profiles(ProfilesCommand::Edit { agent }) => {
            let agent = Agent::parse(&agent)?;
            let file = write_template(&paths, agent)?;
            let editor = std::env::var("EDITOR").unwrap_or_else(|_| "vi".into());
            let status = std::process::Command::new(editor).arg(file).status()?;
            Ok(status.code().unwrap_or(1))
        }
        Command::Profiles(ProfilesCommand::Remove { agent, yes }) => {
            let agent = Agent::parse(&agent)?;
            if !yes {
                bail!("refusing to remove {} without --yes", agent.canonical());
            }
            let file = paths.profile_file(agent);
            if file.exists() {
                std::fs::remove_file(&file)?;
            }
            println!("Removed {}", agent.canonical());
            Ok(0)
        }
        Command::Use { agent, global } => {
            let agent = Agent::parse(&agent)?;
            if global {
                write_default_profile(&paths, agent)?;
                println!("Default agent: {}", agent.canonical());
            } else {
                println!("Run this once, or run `ccs init` to install the shell hook:");
                println!(
                    "eval \"$({} internal env use {})\"",
                    current_binary(),
                    agent.canonical()
                );
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
            println!("Shell hook installed");
            if !hooks_only {
                println!("Next: ccs profiles add ds");
            }
            Ok(0)
        }
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
        Command::PermissionsBypass => {
            let file = crate::permissions::set_bypass_permissions(&std::env::current_dir()?)?;
            println!("Updated {}", file.display());
            Ok(0)
        }
        Command::Update => {
            crate::update::run_update()?;
            Ok(0)
        }
        Command::Help => {
            print_help();
            Ok(0)
        }
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

fn current_binary() -> String {
    std::env::current_exe()
        .ok()
        .map(|path| path.display().to_string())
        .unwrap_or_else(|| "ccs".into())
}
