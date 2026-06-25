use ccs::env::render_shell_exports;
use ccs::glm::GlmPlatform;
use ccs::links::ensure_shared_links;
use ccs::paths::Paths;
use ccs::profile::{Profile, read_default_profile, write_default_profile};
use ccs::provider::Provider;
use tempfile::TempDir;

#[test]
fn writes_and_reads_canonical_default_profile() {
    let home = TempDir::new().unwrap();
    let paths = Paths::from_home(home.path());
    write_default_profile(&paths, Provider::Deepseek).unwrap();
    assert_eq!(
        read_default_profile(&paths).unwrap(),
        Some(Provider::Deepseek)
    );
    let config = std::fs::read_to_string(paths.config_file()).unwrap();
    assert_eq!(config, "default_profile=deepseek\n");
}

#[test]
fn loads_existing_dotenv_profile() {
    let home = TempDir::new().unwrap();
    let paths = Paths::from_home(home.path());
    std::fs::create_dir_all(paths.profiles_dir()).unwrap();
    std::fs::write(
        paths.profile_file(Provider::Anthropic),
        "CLAUDE_CONFIG_DIR=/tmp/anthropic\nANTHROPIC_API_KEY=secret\n",
    )
    .unwrap();

    let profile = Profile::load(&paths, Provider::Anthropic).unwrap();
    assert_eq!(profile.value("CLAUDE_CONFIG_DIR"), Some("/tmp/anthropic"));
    assert_eq!(profile.value("ANTHROPIC_API_KEY"), Some("secret"));
}

#[test]
fn shell_exports_clear_known_vars_and_hide_ccs_internal_keys() {
    let home = TempDir::new().unwrap();
    let paths = Paths::from_home(home.path());
    std::fs::create_dir_all(paths.profiles_dir()).unwrap();
    std::fs::write(
        paths.profile_file(Provider::Anthropic),
        "CLAUDE_CONFIG_DIR=/tmp/anthropic\nCCS_SHARED_PATHS=skills\nANTHROPIC_API_KEY=secret\n",
    )
    .unwrap();

    let profile = Profile::load(&paths, Provider::Anthropic).unwrap();
    let exports = render_shell_exports(&profile, Provider::Anthropic, None).unwrap();
    assert!(exports.contains("unset ANTHROPIC_AUTH_TOKEN"));
    assert!(exports.contains("export CLAUDE_CONFIG_DIR="));
    assert!(exports.contains("export ANTHROPIC_API_KEY="));
    assert!(exports.contains("export CCS_ACTIVE_PROFILE="));
    assert!(!exports.contains("export CCS_SHARED_PATHS="));
}

#[test]
fn glm_shell_exports_derive_zai_api_key_for_vision_mcp() {
    let home = TempDir::new().unwrap();
    let paths = Paths::from_home(home.path());
    std::fs::create_dir_all(paths.profiles_dir()).unwrap();
    std::fs::write(
        paths.profile_file(Provider::Glm),
        "CLAUDE_CONFIG_DIR=/tmp/glm\nANTHROPIC_BASE_URL=https://api.z.ai/api/anthropic\nANTHROPIC_AUTH_TOKEN='glm token'\n",
    )
    .unwrap();

    let profile = Profile::load(&paths, Provider::Glm).unwrap();
    let exports = render_shell_exports(&profile, Provider::Glm, None).unwrap();

    assert!(exports.contains("unset ZAI_API_KEY"));
    assert!(exports.contains("unset Z_AI_API_KEY"));
    assert!(exports.contains("unset Z_AI_MODE"));
    assert!(exports.contains("export ZAI_API_KEY='glm token'"));
    assert!(exports.contains("export Z_AI_MODE='ZAI'"));
    assert!(exports.contains("export Z_AI_VISION_MODEL='glm-5v-turbo'"));
    assert!(exports.contains("export CLAUDE_CODE_AUTO_COMPACT_WINDOW='900000'"));
}

#[test]
fn glm_shell_exports_auto_compact_window_from_percent() {
    let home = TempDir::new().unwrap();
    let paths = Paths::from_home(home.path());
    std::fs::create_dir_all(paths.profiles_dir()).unwrap();
    std::fs::write(
        paths.profile_file(Provider::Glm),
        "CLAUDE_CONFIG_DIR=/tmp/glm\nGLM_ZAI_API_KEY=token\nGLM_CONTEXT_TOKENS=2000000\nGLM_AUTO_COMPACT_PERCENT=80\n",
    )
    .unwrap();

    let profile = Profile::load(&paths, Provider::Glm).unwrap();
    let exports = render_shell_exports(&profile, Provider::Glm, None).unwrap();

    assert!(exports.contains("unset GLM_AUTO_COMPACT_PERCENT"));
    assert!(!exports.contains("export GLM_AUTO_COMPACT_PERCENT="));
    assert!(exports.contains("export CLAUDE_CODE_AUTO_COMPACT_WINDOW='1600000'"));
}

#[test]
fn explicit_zai_api_key_wins_over_glm_auth_token() {
    let home = TempDir::new().unwrap();
    let paths = Paths::from_home(home.path());
    std::fs::create_dir_all(paths.profiles_dir()).unwrap();
    std::fs::write(
        paths.profile_file(Provider::Glm),
        "CLAUDE_CONFIG_DIR=/tmp/glm\nANTHROPIC_BASE_URL=https://api.z.ai/api/anthropic\nANTHROPIC_AUTH_TOKEN=glm-token\nZAI_API_KEY=mcp-token\nZ_AI_MODE=ZHIPU\n",
    )
    .unwrap();

    let profile = Profile::load(&paths, Provider::Glm).unwrap();
    let exports = render_shell_exports(&profile, Provider::Glm, None).unwrap();

    assert!(exports.contains("export ZAI_API_KEY='mcp-token'"));
    assert!(exports.contains("export Z_AI_MODE='ZHIPU'"));
    assert!(!exports.contains("export ZAI_API_KEY='glm-token'"));
    assert!(!exports.contains("export Z_AI_MODE='ZAI'"));
}

#[test]
fn legacy_z_ai_api_key_still_works() {
    let home = TempDir::new().unwrap();
    let paths = Paths::from_home(home.path());
    std::fs::create_dir_all(paths.profiles_dir()).unwrap();
    std::fs::write(
        paths.profile_file(Provider::Glm),
        "CLAUDE_CONFIG_DIR=/tmp/glm\nZ_AI_API_KEY=legacy-token\n",
    )
    .unwrap();

    let profile = Profile::load(&paths, Provider::Glm).unwrap();
    let exports = render_shell_exports(&profile, Provider::Glm, None).unwrap();

    assert!(exports.contains("export ZAI_API_KEY='legacy-token'"));
    assert!(!exports.contains("export Z_AI_API_KEY="));
}

#[test]
fn glm_shell_exports_resolve_domestic_platform_from_profile() {
    let home = TempDir::new().unwrap();
    let paths = Paths::from_home(home.path());
    std::fs::create_dir_all(paths.profiles_dir()).unwrap();
    std::fs::write(
        paths.profile_file(Provider::Glm),
        "CLAUDE_CONFIG_DIR=/tmp/glm\nGLM_PLATFORM=zhipu\nGLM_ZAI_API_KEY=oversea-token\nGLM_ZHIPU_API_KEY=domestic-token\n",
    )
    .unwrap();

    let profile = Profile::load(&paths, Provider::Glm).unwrap();
    let exports = render_shell_exports(&profile, Provider::Glm, None).unwrap();

    assert!(exports.contains("export ANTHROPIC_BASE_URL='https://open.bigmodel.cn/api/anthropic'"));
    assert!(exports.contains("export ANTHROPIC_AUTH_TOKEN='domestic-token'"));
    assert!(exports.contains("export ZAI_API_KEY='domestic-token'"));
    assert!(exports.contains("export Z_AI_MODE='ZHIPU'"));
}

#[test]
fn glm_shell_exports_normalizes_domestic_api_key_alias() {
    let home = TempDir::new().unwrap();
    let paths = Paths::from_home(home.path());
    std::fs::create_dir_all(paths.profiles_dir()).unwrap();
    std::fs::write(
        paths.profile_file(Provider::Glm),
        "CLAUDE_CONFIG_DIR=/tmp/glm\nGLM_PLATFORM=zhipu\nZHIPU_API_KEY=domestic-token\n",
    )
    .unwrap();

    let profile = Profile::load(&paths, Provider::Glm).unwrap();
    let exports = render_shell_exports(&profile, Provider::Glm, None).unwrap();

    assert!(exports.contains("export ZAI_API_KEY='domestic-token'"));
    assert!(!exports.contains("export ZHIPU_API_KEY="));
}

#[test]
fn glm_shell_exports_platform_option_overrides_profile_platform() {
    let home = TempDir::new().unwrap();
    let paths = Paths::from_home(home.path());
    std::fs::create_dir_all(paths.profiles_dir()).unwrap();
    std::fs::write(
        paths.profile_file(Provider::Glm),
        "CLAUDE_CONFIG_DIR=/tmp/glm\nGLM_PLATFORM=zai\nGLM_ZAI_API_KEY=oversea-token\nGLM_ZHIPU_API_KEY=domestic-token\n",
    )
    .unwrap();

    let profile = Profile::load(&paths, Provider::Glm).unwrap();
    let exports = render_shell_exports(&profile, Provider::Glm, Some(GlmPlatform::Zhipu)).unwrap();

    assert!(exports.contains("export ANTHROPIC_BASE_URL='https://open.bigmodel.cn/api/anthropic'"));
    assert!(exports.contains("export ANTHROPIC_AUTH_TOKEN='domestic-token'"));
    assert!(exports.contains("export Z_AI_MODE='ZHIPU'"));
}

#[test]
fn invalid_glm_platform_in_profile_is_rejected() {
    let home = TempDir::new().unwrap();
    let paths = Paths::from_home(home.path());
    std::fs::create_dir_all(paths.profiles_dir()).unwrap();
    std::fs::write(
        paths.profile_file(Provider::Glm),
        "CLAUDE_CONFIG_DIR=/tmp/glm\nGLM_PLATFORM=random\nGLM_ZAI_API_KEY=oversea-token\n",
    )
    .unwrap();

    let profile = Profile::load(&paths, Provider::Glm).unwrap();
    let error = render_shell_exports(&profile, Provider::Glm, None)
        .unwrap_err()
        .to_string();

    assert!(error.contains("unknown GLM platform"));
}

#[test]
fn creates_default_shared_symlinks() {
    let home = TempDir::new().unwrap();
    let shared = home.path().join(".claude");
    std::fs::create_dir_all(shared.join("skills")).unwrap();
    std::fs::write(shared.join("settings.json"), "{}\n").unwrap();
    std::fs::write(shared.join("CLAUDE.md"), "base\n").unwrap();

    let config_dir = home.path().join(".config/ccs/claude/anthropic");
    let paths = Paths::from_home(home.path());
    std::fs::create_dir_all(paths.profiles_dir()).unwrap();
    std::fs::write(
        paths.profile_file(Provider::Anthropic),
        format!(
            "CLAUDE_CONFIG_DIR={}\nCCS_SHARED_CLAUDE_DIR={}\nCCS_SHARED_PATHS=CLAUDE.md,settings.json,skills\nANTHROPIC_API_KEY=secret\n",
            config_dir.display(),
            shared.display()
        ),
    )
    .unwrap();

    let profile = Profile::load(&paths, Provider::Anthropic).unwrap();
    ensure_shared_links(&profile).unwrap();

    assert_eq!(
        std::fs::read_link(config_dir.join("CLAUDE.md")).unwrap(),
        shared.join("CLAUDE.md")
    );
    assert_eq!(
        std::fs::read_link(config_dir.join("settings.json")).unwrap(),
        shared.join("settings.json")
    );
    assert_eq!(
        std::fs::read_link(config_dir.join("skills")).unwrap(),
        shared.join("skills")
    );
}

#[test]
fn backs_up_existing_local_shared_path_before_linking() {
    let home = TempDir::new().unwrap();
    let shared = home.path().join(".claude");
    std::fs::create_dir_all(&shared).unwrap();
    std::fs::write(shared.join("settings.json"), "shared\n").unwrap();
    let config_dir = home.path().join(".config/ccs/claude/glm");
    std::fs::create_dir_all(&config_dir).unwrap();
    std::fs::write(config_dir.join("settings.json"), "local\n").unwrap();

    let paths = Paths::from_home(home.path());
    std::fs::create_dir_all(paths.profiles_dir()).unwrap();
    std::fs::write(
        paths.profile_file(Provider::Glm),
        format!(
            "CLAUDE_CONFIG_DIR={}\nCCS_SHARED_CLAUDE_DIR={}\nCCS_SHARED_PATHS=settings.json\nANTHROPIC_BASE_URL=https://example.test\nANTHROPIC_AUTH_TOKEN=token\n",
            config_dir.display(),
            shared.display()
        ),
    )
    .unwrap();

    let profile = Profile::load(&paths, Provider::Glm).unwrap();
    ensure_shared_links(&profile).unwrap();

    assert_eq!(
        std::fs::read_link(config_dir.join("settings.json")).unwrap(),
        shared.join("settings.json")
    );
    assert_eq!(
        std::fs::read_to_string(config_dir.join(".ccs-local-backup/settings.json")).unwrap(),
        "local\n"
    );
}
