use std::ffi::{OsStr, OsString};

use anyhow::{Result, bail};

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
            hooks_only: contains_arg(&args, "--hooks-only"),
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
    let global = contains_arg(&args[1..], "--global");
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
            yes: contains_arg(args, "--yes"),
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

fn required_arg(args: &[OsString], name: &str) -> Result<String> {
    args.get(1)
        .and_then(|value| value.to_str())
        .map(ToOwned::to_owned)
        .ok_or_else(|| anyhow::anyhow!("missing {name}"))
}

fn contains_arg(args: &[OsString], expected: &str) -> bool {
    args.iter().any(|arg| arg == OsStr::new(expected))
}
