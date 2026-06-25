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
        .stdout(predicate::str::contains(
            "export CCS_ACTIVE_PROFILE='deepseek'",
        ));
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
