mod support;

use assert_cmd::Command;
use predicates::prelude::*;
use support::TestHome;

fn ccs(home: &TestHome) -> Command {
    let mut cmd = Command::cargo_bin("ccs").unwrap();
    cmd.env("CCS_TEST_HOME", home.path());
    for key in [
        "GLM_ZAI_API_KEY",
        "GLM_ZHIPU_API_KEY",
        "Z_AI_API_KEY",
        "ZHIPU_API_KEY",
        "ANTHROPIC_AUTH_TOKEN",
    ] {
        cmd.env_remove(key);
    }
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

fn write_glm(home: &TestHome) {
    std::fs::create_dir_all(home.path().join(".config/ccs/profiles")).unwrap();
    std::fs::write(
        home.path().join(".config/ccs/profiles/glm.env"),
        format!(
            "CLAUDE_CONFIG_DIR={}\nANTHROPIC_BASE_URL=https://api.z.ai/api/anthropic\nANTHROPIC_AUTH_TOKEN=glm-token\n",
            home.path().join(".config/ccs/claude/glm").display()
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
        .stdout(predicate::str::contains("Default provider: deepseek"));
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
fn internal_env_for_glm_writes_profile_scoped_mcp_config() {
    let home = TestHome::new();
    write_glm(&home);

    ccs(&home)
        .args(["internal", "env", "use", "glm"])
        .assert()
        .success()
        .stdout(predicate::str::contains(
            "export Z_AI_VISION_MODEL='glm-5v-turbo'",
        ));

    let config =
        std::fs::read_to_string(home.path().join(".config/ccs/claude/glm/.claude.json")).unwrap();
    assert!(config.contains("\"zai-mcp-server\""));
    assert!(!home.path().join(".claude.json").exists());
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

#[test]
fn init_prepares_deepseek_as_default() {
    let home = TestHome::new();
    ccs(&home)
        .arg("init")
        .assert()
        .success()
        .stdout(predicate::str::contains("Default provider: deepseek"));

    assert!(
        home.path()
            .join(".config/ccs/profiles/deepseek.env")
            .exists()
    );
    let config = std::fs::read_to_string(home.path().join(".config/ccs/config")).unwrap();
    assert_eq!(config, "default_profile=deepseek\n");
}

#[test]
fn init_accepts_agent_to_prepare_as_default() {
    let home = TestHome::new();
    ccs(&home)
        .args(["init", "kimi"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Default provider: kimi"));

    assert!(home.path().join(".config/ccs/profiles/kimi.env").exists());
    let config = std::fs::read_to_string(home.path().join(".config/ccs/config")).unwrap();
    assert_eq!(config, "default_profile=kimi\n");
}

#[test]
fn setup_glm_with_existing_token_prepares_mcp_config() {
    let home = TestHome::new();
    write_glm(&home);

    ccs(&home)
        .args(["setup", "glm"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Default provider: glm"))
        .stdout(predicate::str::contains("GLM MCP configured:"))
        .stdout(predicate::str::contains("GLM vision model: glm-5v-turbo"))
        .stdout(predicate::str::contains("GLM auto compact window: 900000"))
        .stdout(predicate::str::contains("Next: ccs profiles edit glm").not());

    let config =
        std::fs::read_to_string(home.path().join(".config/ccs/claude/glm/.claude.json")).unwrap();
    assert!(config.contains("\"web-search-prime\""));
    assert!(config.contains("\"web-reader\""));
    assert!(config.contains("\"zread\""));
    assert!(config.contains("\"glm-5v-turbo\""));

    let profile =
        std::fs::read_to_string(home.path().join(".config/ccs/profiles/glm.env")).unwrap();
    assert!(profile.contains("GLM_PLATFORM=zai"));
    assert!(profile.contains("GLM_ZAI_API_KEY=glm-token"));
    assert!(profile.contains("GLM_CONTEXT_TOKENS=1000000"));
    assert!(profile.contains("GLM_AUTO_COMPACT_PERCENT=90"));
    assert!(profile.contains("Z_AI_VISION_MODEL=glm-5v-turbo"));
    assert!(profile.contains("ANTHROPIC_DEFAULT_OPUS_MODEL=glm-5.2[1m]"));
    assert!(profile.contains("ANTHROPIC_DEFAULT_HAIKU_MODEL=glm-4.7"));
}

#[test]
fn setup_glm_without_token_reports_pending_runtime_config() {
    let home = TestHome::new();

    ccs(&home)
        .args(["setup", "glm"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Default provider: glm"))
        .stdout(predicate::str::contains("GLM runtime pending:"))
        .stdout(predicate::str::contains("GLM MCP configured:").not());

    assert!(
        !home
            .path()
            .join(".config/ccs/claude/glm/.claude.json")
            .exists()
    );
}

#[test]
fn setup_glm_reconfigure_reads_environment_keys() {
    let home = TestHome::new();
    write_glm(&home);

    ccs(&home)
        .env("Z_AI_API_KEY", "fresh-oversea-token")
        .env("ZHIPU_API_KEY", "fresh-domestic-token")
        .args(["setup", "glm", "-r"])
        .assert()
        .success()
        .stdout(predicate::str::contains(
            "GLM profile refreshed from environment",
        ))
        .stdout(predicate::str::contains("GLM MCP configured:"));

    let profile =
        std::fs::read_to_string(home.path().join(".config/ccs/profiles/glm.env")).unwrap();
    assert!(profile.contains("GLM_PLATFORM=zai"));
    assert!(profile.contains("GLM_ZAI_API_KEY=fresh-oversea-token"));
    assert!(profile.contains("GLM_ZHIPU_API_KEY=fresh-domestic-token"));
    assert!(profile.contains("ANTHROPIC_AUTH_TOKEN=fresh-oversea-token"));

    let config =
        std::fs::read_to_string(home.path().join(".config/ccs/claude/glm/.claude.json")).unwrap();
    assert!(config.contains("\"Authorization\": \"Bearer fresh-oversea-token\""));
}

#[test]
fn setup_glm_migrates_old_default_auto_compact_percent() {
    let home = TestHome::new();
    std::fs::create_dir_all(home.path().join(".config/ccs/profiles")).unwrap();
    std::fs::write(
        home.path().join(".config/ccs/profiles/glm.env"),
        format!(
            "CLAUDE_CONFIG_DIR={}\nANTHROPIC_AUTH_TOKEN=glm-token\nGLM_AUTO_COMPACT_PERCENT=85\n",
            home.path().join(".config/ccs/claude/glm").display()
        ),
    )
    .unwrap();

    ccs(&home).args(["setup", "glm"]).assert().success();

    let profile =
        std::fs::read_to_string(home.path().join(".config/ccs/profiles/glm.env")).unwrap();
    assert!(profile.contains("GLM_AUTO_COMPACT_PERCENT=90"));
}

#[test]
fn setup_glm_platform_short_option_updates_profile_platform() {
    let home = TestHome::new();
    write_glm(&home);

    ccs(&home)
        .args(["setup", "glm", "-p", "zhipu"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Default provider: glm"));

    let profile =
        std::fs::read_to_string(home.path().join(".config/ccs/profiles/glm.env")).unwrap();
    assert!(profile.contains("GLM_PLATFORM=zhipu"));
    assert!(profile.contains("GLM_ZHIPU_API_KEY=glm-token"));
    assert!(profile.contains("ANTHROPIC_BASE_URL=https://open.bigmodel.cn/api/anthropic"));
    assert!(profile.contains("Z_AI_MODE=ZHIPU"));
}

#[test]
fn init_accepts_anthropic_provider() {
    let home = TestHome::new();
    ccs(&home)
        .args(["init", "anthropic"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Default provider: anthropic"));

    assert!(
        home.path()
            .join(".config/ccs/profiles/anthropic.env")
            .exists()
    );
    let config = std::fs::read_to_string(home.path().join(".config/ccs/config")).unwrap();
    assert_eq!(config, "default_profile=anthropic\n");
}
