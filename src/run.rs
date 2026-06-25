use std::ffi::OsString;
use std::io::IsTerminal;
use std::path::Path;
use std::process::Command as ProcessCommand;

use anyhow::{Result, bail};

use crate::cli::{Command, ProfilesCommand};
use crate::env::{KNOWN_ENV_VARS, derived_provider_env, render_shell_exports};
use crate::glm::{GlmPlatform, resolve_glm};
use crate::links::ensure_shared_links;
use crate::mcp::{ensure_provider_mcp, glm_mcp_configured, glm_mcp_file};
use crate::paths::Paths;
use crate::profile::{
    Profile, read_default_profile, write_default_profile, write_glm_defaults, write_template,
};
use crate::provider::Provider;
use crate::shell;

pub fn execute(command: Command) -> Result<i32> {
    let paths = Paths::from_env()?;
    match command {
        Command::LaunchDefault => {
            let Some(provider) = read_default_profile(&paths)? else {
                bail!("No default provider set.\nRun: ccs use ds --global\nSee: ccs profiles");
            };
            launch_provider(&paths, provider, &[], None)
        }
        Command::LaunchProvider {
            provider,
            platform,
            claude_args,
        } => {
            let provider = Provider::parse(&provider)?;
            launch_provider(&paths, provider, &claude_args, platform)
        }
        Command::Profiles(ProfilesCommand::List) => {
            for provider in Provider::all() {
                let file = paths.profile_file(*provider);
                if file.exists() {
                    println!("{} -> {}", provider.canonical(), file.display());
                }
            }
            Ok(0)
        }
        Command::Profiles(ProfilesCommand::Add { provider }) => {
            let file = write_template(&paths, provider)?;
            println!("Added {} profile: {}", provider.canonical(), file.display());
            Ok(0)
        }
        Command::Profiles(ProfilesCommand::Edit { provider }) => {
            let file = write_template(&paths, provider)?;
            edit_profile_file(&file)
        }
        Command::Profiles(ProfilesCommand::Remove { provider, yes }) => {
            if !yes {
                bail!("refusing to remove {} without --yes", provider.canonical());
            }
            let file = paths.profile_file(provider);
            if file.exists() {
                std::fs::remove_file(&file)?;
            }
            println!("Removed {}", provider.canonical());
            Ok(0)
        }
        Command::Use {
            provider,
            platform,
            global,
        } => {
            ensure_platform_allowed(provider, platform)?;
            if global {
                if provider == Provider::Glm
                    && let Some(platform) = platform
                {
                    write_glm_defaults(&paths, Some(platform), false)?;
                }
                write_default_profile(&paths, provider)?;
                println!("Default provider: {}", provider.canonical());
            } else {
                println!("Run this once, or run `ccs init` to install the shell hook:");
                let platform_args = platform_args(platform);
                println!(
                    "eval \"$({} internal env use {}{})\"",
                    current_binary(),
                    provider.canonical(),
                    platform_args
                );
            }
            Ok(0)
        }
        Command::InternalEnv { provider, platform } => {
            ensure_platform_allowed(provider, platform)?;
            let profile = Profile::load(&paths, provider)?;
            ensure_shared_links(&profile)?;
            ensure_provider_mcp(&profile, provider, platform)?;
            print!("{}", render_shell_exports(&profile, provider, platform)?);
            Ok(0)
        }
        Command::Init {
            provider,
            platform,
            hooks_only,
            reconfigure,
        } => {
            ensure_platform_allowed(provider, platform)?;
            if reconfigure && provider != Provider::Glm {
                bail!("--reconfigure is only supported for glm");
            }
            shell::install_hooks(&paths, &current_binary())?;
            println!("Shell hook installed");
            if !hooks_only {
                let file = if provider == Provider::Glm {
                    match platform {
                        Some(platform) => write_glm_defaults(&paths, Some(platform), reconfigure)?,
                        None => write_glm_defaults(&paths, None, reconfigure)?,
                    }
                } else {
                    write_template(&paths, provider)?
                };
                write_default_profile(&paths, provider)?;
                if provider == Provider::Glm && reconfigure {
                    println!("GLM profile refreshed from environment");
                }
                let mut runtime_ready = provider != Provider::Glm;
                let mut loaded = Profile::load(&paths, provider);
                if provider == Provider::Glm && reconfigure && setup_can_open_editor() {
                    println!("Opening GLM profile for reconfigure: {}", file.display());
                    let code = edit_profile_file(&file)?;
                    if code != 0 {
                        return Ok(code);
                    }
                    loaded = Profile::load(&paths, provider);
                } else if loaded.is_err() && provider == Provider::Glm && setup_can_open_editor() {
                    println!("Opening GLM profile for API key: {}", file.display());
                    let code = edit_profile_file(&file)?;
                    if code != 0 {
                        return Ok(code);
                    }
                    loaded = Profile::load(&paths, provider);
                }
                match loaded {
                    Ok(profile) => {
                        ensure_shared_links(&profile)?;
                        if let Some(file) = ensure_provider_mcp(&profile, provider, platform)? {
                            let glm = resolve_glm(&profile, platform)?;
                            runtime_ready = true;
                            println!("GLM MCP configured: {}", file.display());
                            println!("GLM vision model: {}", glm.vision_model);
                            println!("GLM auto compact window: {}", glm.auto_compact_window);
                        }
                    }
                    Err(error) if provider == Provider::Glm => {
                        println!("GLM runtime pending: {error}");
                    }
                    Err(_) => {}
                }
                println!("Profile ready: {}", file.display());
                println!("Default provider: {}", provider.canonical());
                if !runtime_ready {
                    println!("Next: ccs profiles edit {}", provider.canonical());
                }
            }
            Ok(0)
        }
        Command::Status => {
            let active = std::env::var("CCS_ACTIVE_PROFILE").unwrap_or_else(|_| "none".into());
            let default = read_default_profile(&paths)?
                .map(|provider| provider.canonical().to_owned())
                .unwrap_or_else(|| "none".into());
            println!("Active provider: {active}");
            println!("Default provider: {default}");
            println!("Profiles:");
            for provider in Provider::all() {
                let file = paths.profile_file(*provider);
                if file.exists() {
                    println!("  {} -> {}", provider.canonical(), file.display());
                }
            }
            print_glm_status(&paths)?;
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
        Command::Version => {
            println!("ccs {}", env!("CARGO_PKG_VERSION"));
            Ok(0)
        }
    }
}

pub fn launch_provider(
    paths: &Paths,
    provider: Provider,
    args: &[OsString],
    platform: Option<GlmPlatform>,
) -> Result<i32> {
    ensure_platform_allowed(provider, platform)?;
    let profile = Profile::load(paths, provider)?;
    ensure_shared_links(&profile)?;
    ensure_provider_mcp(&profile, provider, platform)?;

    let mut command = ProcessCommand::new("claude");
    command.args(args);
    for key in KNOWN_ENV_VARS {
        command.env_remove(key);
    }
    for (key, value) in profile.iter() {
        if !key.starts_with("CCS_") && !key.starts_with("GLM_") {
            command.env(key, value);
        }
    }
    for (key, value) in derived_provider_env(&profile, provider, platform)? {
        command.env(key, value);
    }
    command.env("CCS_ACTIVE_PROFILE", provider.canonical());

    if std::env::var_os("CCS_TEST_NO_EXEC").is_some() {
        let status = command.status()?;
        return Ok(status.code().unwrap_or(1));
    }

    use std::os::unix::process::CommandExt;
    let error = command.exec();
    Err(error.into())
}

fn ensure_platform_allowed(provider: Provider, platform: Option<GlmPlatform>) -> Result<()> {
    if platform.is_some() && provider != Provider::Glm {
        bail!("--platform is only supported for glm");
    }
    Ok(())
}

fn platform_args(platform: Option<GlmPlatform>) -> String {
    platform
        .map(|platform| format!(" --platform {}", platform.canonical()))
        .unwrap_or_default()
}

fn print_glm_status(paths: &Paths) -> Result<()> {
    if !paths.profile_file(Provider::Glm).exists() {
        return Ok(());
    }
    match Profile::load(paths, Provider::Glm) {
        Ok(profile) => {
            let glm = resolve_glm(&profile, None)?;
            let mcp_state = if glm_mcp_configured(&profile)? {
                "configured"
            } else {
                "pending"
            };
            println!("GLM platform: {}", glm.platform.canonical());
            println!("GLM vision model: {}", glm.vision_model);
            println!("GLM auto compact window: {}", glm.auto_compact_window);
            println!(
                "GLM MCP: {} -> {}",
                mcp_state,
                glm_mcp_file(&profile)?.display()
            );
        }
        Err(error) => {
            println!("GLM runtime pending: {error}");
        }
    }
    Ok(())
}

fn edit_profile_file(file: &Path) -> Result<i32> {
    let editor = std::env::var("EDITOR").unwrap_or_else(|_| "vi".into());
    let status = std::process::Command::new(editor).arg(file).status()?;
    Ok(status.code().unwrap_or(1))
}

fn setup_can_open_editor() -> bool {
    std::io::stdin().is_terminal() && std::io::stdout().is_terminal()
}

fn current_binary() -> String {
    std::env::current_exe()
        .ok()
        .map(|path| path.display().to_string())
        .unwrap_or_else(|| "ccs".into())
}
