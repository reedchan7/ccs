mod support;

use assert_cmd::Command;
use predicates::prelude::*;
use support::TestHome;

fn ccs(home: &TestHome) -> Command {
    let mut cmd = Command::cargo_bin("ccs").unwrap();
    let path = format!("{}:{}", home.bin().display(), std::env::var("PATH").unwrap());
    cmd.env("CCS_TEST_HOME", home.path())
        .env("CCS_TEST_NO_EXEC", "1")
        .env("PATH", path);
    cmd
}

#[test]
fn bare_ccs_without_default_prints_next_step() {
    let home = TestHome::new();
    ccs(&home)
        .assert()
        .failure()
        .stderr(predicate::str::contains("No default agent set"));
}

#[test]
fn bare_ccs_runs_default_agent() {
    let home = TestHome::new();
    home.write_fake_claude();
    std::fs::create_dir_all(home.path().join(".config/ccs/profiles")).unwrap();
    std::fs::write(
        home.path().join(".config/ccs/profiles/deepseek.env"),
        format!(
            "CLAUDE_CONFIG_DIR={}\nANTHROPIC_BASE_URL=https://api.deepseek.com/anthropic\nANTHROPIC_AUTH_TOKEN=token\n",
            home.path().join(".config/ccs/claude/deepseek").display()
        ),
    )
    .unwrap();
    std::fs::create_dir_all(home.path().join(".config/ccs")).unwrap();
    std::fs::write(
        home.path().join(".config/ccs/config"),
        "default_profile=deepseek\n",
    )
    .unwrap();

    ccs(&home)
        .assert()
        .success()
        .stdout(predicate::str::contains("CCS_ACTIVE_PROFILE=deepseek"));
}

#[test]
fn ccs_ds_runs_deepseek_and_passes_args() {
    let home = TestHome::new();
    home.write_fake_claude();
    std::fs::create_dir_all(home.path().join(".config/ccs/profiles")).unwrap();
    std::fs::write(
        home.path().join(".config/ccs/profiles/deepseek.env"),
        format!(
            "CLAUDE_CONFIG_DIR={}\nANTHROPIC_BASE_URL=https://api.deepseek.com/anthropic\nANTHROPIC_AUTH_TOKEN=token\n",
            home.path().join(".config/ccs/claude/deepseek").display()
        ),
    )
    .unwrap();

    ccs(&home)
        .args(["ds", "--print", "hello"])
        .assert()
        .success()
        .stdout(predicate::str::contains("CCS_ACTIVE_PROFILE=deepseek"))
        .stdout(predicate::str::contains("ARGS=--print hello "));
}
