use std::ffi::OsString;

use predicates::prelude::*;

use ccs::cli::{Command, ProfilesCommand, parse};
use ccs::glm::GlmPlatform;
use ccs::provider::Provider;

fn os_vec(items: &[&str]) -> Vec<OsString> {
    items.iter().map(OsString::from).collect()
}

#[test]
fn bare_ccs_launches_default_provider() {
    let command = parse(os_vec(&["ccs"])).unwrap();
    assert_eq!(command, Command::LaunchDefault);
}

#[test]
fn first_unknown_token_is_provider_entry_with_passthrough_args() {
    let command = parse(os_vec(&["ccs", "ds", "--print", "hello"])).unwrap();
    assert_eq!(
        command,
        Command::LaunchProvider {
            provider: "ds".into(),
            platform: None,
            claude_args: os_vec(&["--print", "hello"]),
        }
    );
}

#[test]
fn provider_launch_consumes_glm_platform_short_option() {
    let command = parse(os_vec(&["ccs", "glm", "-p", "zhipu", "--print", "hello"])).unwrap();
    assert_eq!(
        command,
        Command::LaunchProvider {
            provider: "glm".into(),
            platform: Some(GlmPlatform::Zhipu),
            claude_args: os_vec(&["--print", "hello"]),
        }
    );
}

#[test]
fn provider_launch_consumes_glm_platform_long_option() {
    let command = parse(os_vec(&[
        "ccs",
        "glm",
        "--platform=zai",
        "--print",
        "hello",
    ]))
    .unwrap();
    assert_eq!(
        command,
        Command::LaunchProvider {
            provider: "glm".into(),
            platform: Some(GlmPlatform::Zai),
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
            provider: Provider::Deepseek,
            platform: None,
            global: true,
        }
    );
}

#[test]
fn use_supports_glm_platform_short_option() {
    let command = parse(os_vec(&["ccs", "use", "glm", "-p", "zhipu"])).unwrap();
    assert_eq!(
        command,
        Command::Use {
            provider: Provider::Glm,
            platform: Some(GlmPlatform::Zhipu),
            global: false,
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

#[test]
fn setup_is_init_alias() {
    let command = parse(os_vec(&["ccs", "setup"])).unwrap();
    assert_eq!(
        command,
        Command::Init {
            provider: Provider::Deepseek,
            platform: None,
            hooks_only: false,
            reconfigure: false,
        }
    );

    let command = parse(os_vec(&["ccs", "init"])).unwrap();
    assert_eq!(
        command,
        Command::Init {
            provider: Provider::Deepseek,
            platform: None,
            hooks_only: false,
            reconfigure: false,
        }
    );
}

#[test]
fn setup_supports_glm_platform_short_option() {
    let command = parse(os_vec(&["ccs", "setup", "glm", "-p", "zhipu"])).unwrap();
    assert_eq!(
        command,
        Command::Init {
            provider: Provider::Glm,
            platform: Some(GlmPlatform::Zhipu),
            hooks_only: false,
            reconfigure: false,
        }
    );
}

#[test]
fn setup_supports_glm_reconfigure_short_option() {
    let command = parse(os_vec(&["ccs", "setup", "glm", "-r", "-p", "zai"])).unwrap();
    assert_eq!(
        command,
        Command::Init {
            provider: Provider::Glm,
            platform: Some(GlmPlatform::Zai),
            hooks_only: false,
            reconfigure: true,
        }
    );
}

#[test]
fn provider_args_reject_unknown_values_with_choices() {
    let error = parse(os_vec(&["ccs", "use", "random"])).unwrap_err();
    let message = error.to_string();

    assert!(predicate::str::contains("possible values").eval(&message));
    assert!(predicate::str::contains("anthropic").eval(&message));
    assert!(predicate::str::contains("deepseek").eval(&message));
    assert!(predicate::str::contains("kimi").eval(&message));
    assert!(!predicate::str::contains("max").eval(&message));
    assert!(!predicate::str::contains("api").eval(&message));
}

#[test]
fn version_command_shows_version() {
    assert_eq!(
        parse(os_vec(&["ccs", "version"])).unwrap(),
        Command::Version
    );
}

#[test]
fn self_update_is_update_alias() {
    assert_eq!(
        parse(os_vec(&["ccs", "self-update"])).unwrap(),
        Command::Update
    );
}
