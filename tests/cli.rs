use std::ffi::OsString;

use ccs::cli::{Command, ProfilesCommand, parse};

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
