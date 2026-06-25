use ccs::glm::GlmPlatform;
use ccs::provider::Provider;
use std::collections::HashMap;

#[test]
fn ds_is_deepseek_alias() {
    let provider = Provider::parse("ds").unwrap();
    assert_eq!(provider.canonical(), "deepseek");
}

#[test]
fn glm_platform_accepts_domestic_and_international_aliases() {
    assert_eq!(
        GlmPlatform::parse("international").unwrap(),
        GlmPlatform::Zai
    );
    assert_eq!(GlmPlatform::parse("domestic").unwrap(), GlmPlatform::Zhipu);
    assert!(GlmPlatform::parse("random").is_err());
}

#[test]
fn built_in_profiles_are_api_providers_only() {
    let names: Vec<_> = Provider::all()
        .iter()
        .map(|provider| provider.canonical())
        .collect();
    assert_eq!(names, ["anthropic", "glm", "mimo", "deepseek", "kimi"]);
}

#[test]
fn anthropic_template_uses_api_key() {
    let home = std::path::Path::new("/tmp/home");
    let profile = Provider::parse("anthropic").unwrap().template(home);
    let keys: Vec<_> = profile.iter().map(|(key, _)| key.as_str()).collect();
    assert!(keys.contains(&"ANTHROPIC_API_KEY"));
    assert!(!keys.contains(&"ANTHROPIC_AUTH_TOKEN"));
}

#[test]
fn deepseek_template_contains_required_model_env() {
    let home = std::path::Path::new("/tmp/home");
    let profile = Provider::parse("deepseek").unwrap().template(home);
    let keys: Vec<_> = profile.iter().map(|(key, _)| key.as_str()).collect();
    assert!(keys.contains(&"ANTHROPIC_BASE_URL"));
    assert!(keys.contains(&"ANTHROPIC_AUTH_TOKEN"));
    assert!(keys.contains(&"ANTHROPIC_DEFAULT_HAIKU_MODEL"));
    assert!(keys.contains(&"CLAUDE_CODE_SUBAGENT_MODEL"));
}

#[test]
fn glm_template_matches_current_claude_code_model_config() {
    let home = std::path::Path::new("/tmp/home");
    let profile = Provider::parse("glm").unwrap().template(home);
    let values: HashMap<_, _> = profile
        .iter()
        .map(|(key, value)| (key.as_str(), value.as_str()))
        .collect();

    assert_eq!(
        values.get("ANTHROPIC_BASE_URL"),
        Some(&"https://api.z.ai/api/anthropic")
    );
    assert_eq!(values.get("ANTHROPIC_AUTH_TOKEN"), Some(&""));
    assert_eq!(
        values.get("ANTHROPIC_DEFAULT_OPUS_MODEL"),
        Some(&"glm-5.2[1m]")
    );
    assert_eq!(
        values.get("ANTHROPIC_DEFAULT_SONNET_MODEL"),
        Some(&"glm-5.2[1m]")
    );
    assert_eq!(
        values.get("ANTHROPIC_DEFAULT_HAIKU_MODEL"),
        Some(&"glm-4.7")
    );
    assert_eq!(values.get("GLM_CONTEXT_TOKENS"), Some(&"1000000"));
    assert_eq!(values.get("GLM_AUTO_COMPACT_PERCENT"), Some(&"90"));
    assert!(!values.contains_key("CLAUDE_CODE_AUTO_COMPACT_WINDOW"));
    assert_eq!(values.get("API_TIMEOUT_MS"), Some(&"3000000"));
    assert_eq!(
        values.get("CLAUDE_CODE_DISABLE_NONESSENTIAL_TRAFFIC"),
        Some(&"1")
    );
    assert_eq!(values.get("Z_AI_VISION_MODEL"), Some(&"glm-5v-turbo"));
}

#[test]
fn glm_template_contains_official_vision_mcp_mode() {
    let home = std::path::Path::new("/tmp/home");
    let profile = Provider::parse("glm").unwrap().template(home);
    let values: HashMap<_, _> = profile
        .iter()
        .map(|(key, value)| (key.as_str(), value.as_str()))
        .collect();

    assert_eq!(values.get("Z_AI_MODE"), Some(&"ZAI"));
    assert!(!values.contains_key("ZAI_API_KEY"));
    assert!(!values.contains_key("Z_AI_API_KEY"));
}

#[test]
fn unknown_provider_is_rejected() {
    let error = Provider::parse("random").unwrap_err().to_string();
    assert!(error.contains("unknown provider"));
}

#[test]
fn subscription_and_generic_api_names_are_rejected() {
    assert!(Provider::parse("max").is_err());
    assert!(Provider::parse("api").is_err());
}
