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
