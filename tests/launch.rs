mod support;

use assert_cmd::Command;
use predicates::prelude::*;
use serde_json::json;
use support::TestHome;

fn ccs(home: &TestHome) -> Command {
    let mut cmd = Command::cargo_bin("ccs").unwrap();
    let path = format!(
        "{}:{}",
        home.bin().display(),
        std::env::var("PATH").unwrap()
    );
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
        .stderr(predicate::str::contains("No default provider set"));
}

#[test]
fn bare_ccs_runs_default_provider() {
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

#[test]
fn ccs_glm_exports_vision_mcp_env_to_claude() {
    let home = TestHome::new();
    home.write_fake_claude();
    std::fs::create_dir_all(home.path().join(".config/ccs/profiles")).unwrap();
    std::fs::write(
        home.path().join(".config/ccs/profiles/glm.env"),
        format!(
            "CLAUDE_CONFIG_DIR={}\nANTHROPIC_BASE_URL=https://api.z.ai/api/anthropic\nANTHROPIC_AUTH_TOKEN=glm-token\n",
            home.path().join(".config/ccs/claude/glm").display()
        ),
    )
    .unwrap();

    ccs(&home)
        .args(["glm", "--print", "vision"])
        .assert()
        .success()
        .stdout(predicate::str::contains("CCS_ACTIVE_PROFILE=glm"))
        .stdout(predicate::str::contains("ANTHROPIC_AUTH_TOKEN=glm-token"))
        .stdout(predicate::str::contains("ZAI_API_KEY=glm-token"))
        .stdout(predicate::str::contains("Z_AI_MODE=ZAI"));
}

#[test]
fn ccs_glm_launch_normalizes_legacy_zai_key() {
    let home = TestHome::new();
    home.write_fake_claude();
    std::fs::create_dir_all(home.path().join(".config/ccs/profiles")).unwrap();
    std::fs::write(
        home.path().join(".config/ccs/profiles/glm.env"),
        format!(
            "CLAUDE_CONFIG_DIR={}\nZ_AI_API_KEY=legacy-token\n",
            home.path().join(".config/ccs/claude/glm").display()
        ),
    )
    .unwrap();

    ccs(&home)
        .args(["glm", "--print", "vision"])
        .assert()
        .success()
        .stdout(predicate::str::contains("ZAI_API_KEY=legacy-token"))
        .stdout(predicate::str::contains("Z_AI_API_KEY=\n"));
}

#[test]
fn ccs_glm_platform_option_uses_domestic_key_and_endpoints() {
    let home = TestHome::new();
    home.write_fake_claude();
    let claude_dir = home.path().join(".config/ccs/claude/glm");
    std::fs::create_dir_all(home.path().join(".config/ccs/profiles")).unwrap();
    std::fs::write(
        home.path().join(".config/ccs/profiles/glm.env"),
        format!(
            "CLAUDE_CONFIG_DIR={}\nGLM_PLATFORM=zai\nGLM_ZAI_API_KEY=oversea-token\nGLM_ZHIPU_API_KEY=domestic-token\n",
            claude_dir.display()
        ),
    )
    .unwrap();

    ccs(&home)
        .args(["glm", "-p", "zhipu", "--print", "vision"])
        .assert()
        .success()
        .stdout(predicate::str::contains("CCS_ACTIVE_PROFILE=glm"))
        .stdout(predicate::str::contains(
            "ANTHROPIC_BASE_URL=https://open.bigmodel.cn/api/anthropic",
        ))
        .stdout(predicate::str::contains(
            "ANTHROPIC_AUTH_TOKEN=domestic-token",
        ))
        .stdout(predicate::str::contains("ZAI_API_KEY=domestic-token"))
        .stdout(predicate::str::contains("Z_AI_MODE=ZHIPU"))
        .stdout(predicate::str::contains("ARGS=--print vision "));

    let config: serde_json::Value =
        serde_json::from_str(&std::fs::read_to_string(claude_dir.join(".claude.json")).unwrap())
            .unwrap();
    assert_eq!(
        config["mcpServers"]["web-search-prime"]["url"],
        json!("https://open.bigmodel.cn/api/mcp/web_search_prime/mcp")
    );
    assert_eq!(
        config["mcpServers"]["zai-mcp-server"]["env"]["Z_AI_MODE"],
        json!("ZHIPU")
    );
    assert_eq!(
        config["mcpServers"]["web-reader"]["headers"]["Authorization"],
        json!("Bearer domestic-token")
    );
}

#[test]
fn ccs_glm_writes_profile_scoped_zai_mcp_config() {
    let home = TestHome::new();
    home.write_fake_claude();
    let claude_dir = home.path().join(".config/ccs/claude/glm");
    std::fs::create_dir_all(home.path().join(".config/ccs/profiles")).unwrap();
    std::fs::create_dir_all(&claude_dir).unwrap();
    std::fs::write(
        claude_dir.join(".claude.json"),
        json!({
            "firstStartTime": "kept",
            "mcpServers": {
                "custom": {
                    "type": "stdio",
                    "command": "custom"
                }
            }
        })
        .to_string(),
    )
    .unwrap();
    std::fs::write(
        home.path().join(".config/ccs/profiles/glm.env"),
        format!(
            "CLAUDE_CONFIG_DIR={}\nANTHROPIC_BASE_URL=https://api.z.ai/api/anthropic\nANTHROPIC_AUTH_TOKEN=glm-token\n",
            claude_dir.display()
        ),
    )
    .unwrap();

    ccs(&home).arg("glm").assert().success();

    assert!(!home.path().join(".claude.json").exists());
    let config: serde_json::Value =
        serde_json::from_str(&std::fs::read_to_string(claude_dir.join(".claude.json")).unwrap())
            .unwrap();
    assert_eq!(config["firstStartTime"], json!("kept"));
    assert_eq!(config["mcpServers"]["custom"]["command"], json!("custom"));
    assert_eq!(
        config["mcpServers"]["zai-mcp-server"]["env"]["Z_AI_VISION_MODEL"],
        json!("glm-5v-turbo")
    );
    assert_eq!(
        config["mcpServers"]["web-search-prime"]["url"],
        json!("https://api.z.ai/api/mcp/web_search_prime/mcp")
    );
    assert_eq!(
        config["mcpServers"]["web-reader"]["url"],
        json!("https://api.z.ai/api/mcp/web_reader/mcp")
    );
    assert_eq!(
        config["mcpServers"]["zread"]["url"],
        json!("https://api.z.ai/api/mcp/zread/mcp")
    );
    assert_eq!(
        config["mcpServers"]["web-reader"]["headers"]["Authorization"],
        json!("Bearer glm-token")
    );
}
