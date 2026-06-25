use std::ffi::OsString;

use clap::error::ErrorKind;
use clap::{Args, Parser, Subcommand};

use crate::glm::GlmPlatform;
use crate::provider::Provider;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Command {
    LaunchDefault,
    LaunchProvider {
        provider: String,
        platform: Option<GlmPlatform>,
        claude_args: Vec<OsString>,
    },
    Use {
        provider: Provider,
        platform: Option<GlmPlatform>,
        global: bool,
    },
    Profiles(ProfilesCommand),
    Init {
        provider: Provider,
        platform: Option<GlmPlatform>,
        hooks_only: bool,
        reconfigure: bool,
    },
    Status,
    Update,
    PermissionsBypass,
    InternalEnv {
        provider: Provider,
        platform: Option<GlmPlatform>,
    },
    Version,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ProfilesCommand {
    List,
    Edit { provider: Provider },
    Add { provider: Provider },
    Remove { provider: Provider, yes: bool },
}

#[derive(Debug, Parser)]
#[command(
    name = "ccs",
    version,
    about = "Claude Code session switcher",
    after_help = "Providers: anthropic, glm, mimo, deepseek/ds, kimi\n\nExamples:\n  ccs\n  ccs ds\n  ccs kimi --print hello\n  ccs setup kimi\n  ccs profiles ls"
)]
struct Cli {
    #[command(subcommand)]
    command: Option<CliCommand>,
}

#[derive(Debug, Subcommand)]
enum CliCommand {
    /// Switch the active provider in this shell, or the default provider with --global
    #[command(name = "use")]
    Use(UseArgs),
    /// Manage provider profiles
    Profiles(ProfilesArgs),
    /// Install the shell hook and prepare a provider as the default
    #[command(name = "setup", visible_alias = "init")]
    Setup(SetupArgs),
    /// Show current active/default provider and configured profiles
    Status,
    /// Manage Claude permissions for the current project
    Permissions(PermissionsArgs),
    /// Update ccs from GitHub Releases
    #[command(visible_alias = "self-update")]
    Update,
    /// Print version information
    Version,
    #[command(name = "internal", hide = true)]
    Internal(InternalArgs),
    #[command(external_subcommand)]
    Provider(Vec<OsString>),
}

#[derive(Debug, Args)]
struct UseArgs {
    #[arg(value_enum, value_name = "PROVIDER")]
    provider: Provider,
    #[arg(short = 'p', long = "platform", value_enum, value_name = "PLATFORM")]
    platform: Option<GlmPlatform>,
    #[arg(long)]
    global: bool,
}

#[derive(Debug, Args)]
struct SetupArgs {
    #[arg(value_enum, value_name = "PROVIDER")]
    provider: Option<Provider>,
    #[arg(short = 'p', long = "platform", value_enum, value_name = "PLATFORM")]
    platform: Option<GlmPlatform>,
    #[arg(long)]
    hooks_only: bool,
    #[arg(short = 'r', long)]
    reconfigure: bool,
}

#[derive(Debug, Args)]
struct ProfilesArgs {
    #[command(subcommand)]
    command: Option<ProfilesSubcommand>,
}

#[derive(Debug, Subcommand)]
enum ProfilesSubcommand {
    /// List configured profiles
    #[command(visible_alias = "ls")]
    List,
    /// Create a built-in profile stub
    Add(ProviderArg),
    /// Edit a profile file
    Edit(ProviderArg),
    /// Remove a profile file
    #[command(visible_alias = "rm")]
    Remove(RemoveProfileArgs),
}

#[derive(Debug, Args)]
struct ProviderArg {
    #[arg(value_enum, value_name = "PROVIDER")]
    provider: Provider,
}

#[derive(Debug, Args)]
struct RemoveProfileArgs {
    #[arg(value_enum, value_name = "PROVIDER")]
    provider: Provider,
    #[arg(long)]
    yes: bool,
}

#[derive(Debug, Args)]
struct PermissionsArgs {
    #[command(subcommand)]
    command: PermissionsSubcommand,
}

#[derive(Debug, Subcommand)]
enum PermissionsSubcommand {
    /// Set Claude Code project permissions to bypass prompts
    Bypass,
}

#[derive(Debug, Args)]
struct InternalArgs {
    #[command(subcommand)]
    command: InternalSubcommand,
}

#[derive(Debug, Subcommand)]
enum InternalSubcommand {
    Env(InternalEnvArgs),
}

#[derive(Debug, Args)]
struct InternalEnvArgs {
    #[command(subcommand)]
    command: InternalEnvSubcommand,
}

#[derive(Debug, Subcommand)]
enum InternalEnvSubcommand {
    Use(InternalUseArgs),
}

#[derive(Debug, Args)]
struct InternalUseArgs {
    #[arg(value_enum, value_name = "PROVIDER")]
    provider: Provider,
    #[arg(short = 'p', long = "platform", value_enum, value_name = "PLATFORM")]
    platform: Option<GlmPlatform>,
}

pub fn parse<I, S>(args: I) -> std::result::Result<Command, clap::Error>
where
    I: IntoIterator<Item = S>,
    S: Into<OsString> + Clone,
{
    let cli = Cli::try_parse_from(args)?;
    cli.into_command()
}

impl Cli {
    fn into_command(self) -> std::result::Result<Command, clap::Error> {
        Ok(match self.command {
            None => Command::LaunchDefault,
            Some(CliCommand::Use(args)) => Command::Use {
                provider: args.provider,
                platform: args.platform,
                global: args.global,
            },
            Some(CliCommand::Profiles(args)) => {
                Command::Profiles(args.command.map_or(ProfilesCommand::List, Into::into))
            }
            Some(CliCommand::Setup(args)) => Command::Init {
                provider: args.provider.unwrap_or(Provider::Deepseek),
                platform: args.platform,
                hooks_only: args.hooks_only,
                reconfigure: args.reconfigure,
            },
            Some(CliCommand::Status) => Command::Status,
            Some(CliCommand::Permissions(args)) => match args.command {
                PermissionsSubcommand::Bypass => Command::PermissionsBypass,
            },
            Some(CliCommand::Update) => Command::Update,
            Some(CliCommand::Version) => Command::Version,
            Some(CliCommand::Internal(args)) => match args.command {
                InternalSubcommand::Env(args) => match args.command {
                    InternalEnvSubcommand::Use(args) => Command::InternalEnv {
                        provider: args.provider,
                        platform: args.platform,
                    },
                },
            },
            Some(CliCommand::Provider(values)) => external_provider(values)?,
        })
    }
}

impl From<ProfilesSubcommand> for ProfilesCommand {
    fn from(value: ProfilesSubcommand) -> Self {
        match value {
            ProfilesSubcommand::List => Self::List,
            ProfilesSubcommand::Add(args) => Self::Add {
                provider: args.provider,
            },
            ProfilesSubcommand::Edit(args) => Self::Edit {
                provider: args.provider,
            },
            ProfilesSubcommand::Remove(args) => Self::Remove {
                provider: args.provider,
                yes: args.yes,
            },
        }
    }
}

fn external_provider(values: Vec<OsString>) -> std::result::Result<Command, clap::Error> {
    let mut values = values.into_iter();
    let Some(provider) = values.next() else {
        return Ok(Command::LaunchDefault);
    };
    let mut platform = None;
    let mut claude_args = Vec::new();
    let mut passthrough = false;
    while let Some(value) = values.next() {
        if passthrough {
            claude_args.push(value);
            continue;
        }

        let raw = value.to_string_lossy();
        if raw == "--" {
            passthrough = true;
        } else if raw == "-p" || raw == "--platform" {
            let Some(value) = values.next() else {
                return Err(clap::Error::raw(
                    ErrorKind::MissingRequiredArgument,
                    "--platform requires a value",
                ));
            };
            platform = Some(parse_platform(&value)?);
        } else if let Some(value) = raw.strip_prefix("--platform=") {
            platform = Some(parse_platform_value(value)?);
        } else {
            claude_args.push(value);
        }
    }

    Ok(Command::LaunchProvider {
        provider: provider.to_string_lossy().into_owned(),
        platform,
        claude_args,
    })
}

fn parse_platform(value: &OsString) -> std::result::Result<GlmPlatform, clap::Error> {
    parse_platform_value(&value.to_string_lossy())
}

fn parse_platform_value(value: &str) -> std::result::Result<GlmPlatform, clap::Error> {
    GlmPlatform::parse(value)
        .map_err(|error| clap::Error::raw(ErrorKind::InvalidValue, error.to_string()))
}
