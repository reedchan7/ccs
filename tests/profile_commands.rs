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
    assert!(
        home.path()
            .join(".config/ccs/profiles/deepseek.env")
            .exists()
    );
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
    std::fs::write(
        home.path().join(".config/ccs/config"),
        "default_profile=deepseek\n",
    )
    .unwrap();

    ccs(&home)
        .arg("status")
        .assert()
        .success()
        .stdout(predicate::str::contains("Default agent: deepseek"))
        .stdout(predicate::str::contains("deepseek"))
        .stdout(predicate::str::contains("secret").not());
}
