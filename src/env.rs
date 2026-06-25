use anyhow::{Result, bail};

use crate::glm::{GlmPlatform, resolve_glm};
use crate::profile::Profile;
use crate::provider::Provider;

pub const KNOWN_ENV_VARS: &[&str] = &[
    "CLAUDE_CONFIG_DIR",
    "CCS_SHARED_CLAUDE_DIR",
    "CCS_SHARED_PATHS",
    "ANTHROPIC_API_KEY",
    "ANTHROPIC_AUTH_TOKEN",
    "ANTHROPIC_BASE_URL",
    "ANTHROPIC_MODEL",
    "ANTHROPIC_DEFAULT_OPUS_MODEL",
    "ANTHROPIC_DEFAULT_SONNET_MODEL",
    "ANTHROPIC_DEFAULT_HAIKU_MODEL",
    "ENABLE_TOOL_SEARCH",
    "CLAUDE_CODE_DISABLE_EXPERIMENTAL_BETAS",
    "CLAUDE_CODE_SUBAGENT_MODEL",
    "CLAUDE_CODE_EFFORT_LEVEL",
    "API_TIMEOUT_MS",
    "CLAUDE_CODE_AUTO_COMPACT_WINDOW",
    "CLAUDE_CODE_DISABLE_NONESSENTIAL_TRAFFIC",
    "GLM_PLATFORM",
    "GLM_ZAI_API_KEY",
    "GLM_ZHIPU_API_KEY",
    "GLM_CONTEXT_TOKENS",
    "GLM_AUTO_COMPACT_PERCENT",
    "Z_AI_API_KEY",
    "ZHIPU_API_KEY",
    "Z_AI_MODE",
    "Z_AI_VISION_MODEL",
    "CCS_ACTIVE_PROFILE",
];

pub fn render_shell_exports(
    profile: &Profile,
    provider: Provider,
    platform: Option<GlmPlatform>,
) -> Result<String> {
    let mut output = String::new();
    for key in KNOWN_ENV_VARS {
        output.push_str(&format!("unset {key}\n"));
    }
    for (key, value) in profile.iter() {
        if key.starts_with("CCS_") || key.starts_with("GLM_") {
            continue;
        }
        output.push_str(&format!("export {key}={}\n", shell_quote(value)));
    }
    for (key, value) in derived_provider_env(profile, provider, platform)? {
        output.push_str(&format!("export {key}={}\n", shell_quote(&value)));
    }
    output.push_str(&format!(
        "export CCS_ACTIVE_PROFILE={}\n",
        shell_quote(provider.canonical())
    ));
    Ok(output)
}

pub fn derived_provider_env(
    profile: &Profile,
    provider: Provider,
    platform: Option<GlmPlatform>,
) -> Result<Vec<(&'static str, String)>> {
    if provider != Provider::Glm {
        if platform.is_some() {
            bail!("--platform is only supported for glm");
        }
        return Ok(Vec::new());
    }

    let glm = resolve_glm(profile, platform)?;
    Ok(vec![
        ("ANTHROPIC_BASE_URL", glm.anthropic_base_url.to_owned()),
        ("ANTHROPIC_AUTH_TOKEN", glm.auth_token.clone()),
        ("Z_AI_API_KEY", glm.auth_token),
        ("Z_AI_MODE", glm.z_ai_mode.to_owned()),
        ("Z_AI_VISION_MODEL", glm.vision_model),
        ("CLAUDE_CODE_AUTO_COMPACT_WINDOW", glm.auto_compact_window),
    ])
}

fn shell_quote(value: &str) -> String {
    format!("'{}'", value.replace('\'', "'\\''"))
}
