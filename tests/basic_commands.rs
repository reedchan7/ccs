use assert_cmd::Command;
use predicates::prelude::*;

#[test]
fn version_flag_prints_package_version() {
    let mut cmd = Command::cargo_bin("ccs").unwrap();
    cmd.arg("-V")
        .assert()
        .success()
        .stdout(predicate::str::contains(concat!(
            "ccs ",
            env!("CARGO_PKG_VERSION")
        )));
}

#[test]
fn help_uses_standard_cli_sections() {
    let mut cmd = Command::cargo_bin("ccs").unwrap();
    cmd.arg("-h")
        .assert()
        .success()
        .stdout(predicate::str::contains("Commands:"))
        .stdout(predicate::str::contains("Options:"))
        .stdout(predicate::str::contains("setup"))
        .stdout(predicate::str::contains("Providers: anthropic"))
        .stdout(predicate::str::contains("max").not())
        .stdout(predicate::str::contains("-V, --version"));
}

#[test]
fn profiles_help_is_scoped_to_profiles() {
    let mut cmd = Command::cargo_bin("ccs").unwrap();
    cmd.args(["profiles", "-h"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Commands:"))
        .stdout(predicate::str::contains("add"))
        .stdout(predicate::str::contains("edit"))
        .stdout(predicate::str::contains("remove"));
}
