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
