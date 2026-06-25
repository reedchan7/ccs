use anyhow::{Result, bail};
use clap::ValueEnum;

use crate::profile::Profile;
use crate::provider::GLM_VISION_MODEL;

const DEFAULT_CONTEXT_TOKENS: &str = "1000000";
const DEFAULT_AUTO_COMPACT_PERCENT: &str = "90";

#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum)]
pub enum GlmPlatform {
    #[value(
        alias = "oversea",
        alias = "overseas",
        alias = "global",
        alias = "intl",
        alias = "international"
    )]
    Zai,
    #[value(alias = "bigmodel", alias = "cn", alias = "domestic")]
    Zhipu,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResolvedGlm {
    pub platform: GlmPlatform,
    pub anthropic_base_url: &'static str,
    pub mcp_base_url: &'static str,
    pub z_ai_mode: &'static str,
    pub auth_token: String,
    pub vision_model: String,
    pub auto_compact_window: String,
}

impl GlmPlatform {
    pub fn parse(value: &str) -> Result<Self> {
        match value {
            "zai" | "oversea" | "overseas" | "global" | "intl" | "international" => Ok(Self::Zai),
            "zhipu" | "bigmodel" | "cn" | "domestic" => Ok(Self::Zhipu),
            other => bail!("unknown GLM platform '{other}'. expected zai or zhipu"),
        }
    }

    pub fn canonical(self) -> &'static str {
        match self {
            Self::Zai => "zai",
            Self::Zhipu => "zhipu",
        }
    }

    pub fn anthropic_base_url(self) -> &'static str {
        match self {
            Self::Zai => "https://api.z.ai/api/anthropic",
            Self::Zhipu => "https://open.bigmodel.cn/api/anthropic",
        }
    }

    pub fn mcp_base_url(self) -> &'static str {
        match self {
            Self::Zai => "https://api.z.ai/api/mcp",
            Self::Zhipu => "https://open.bigmodel.cn/api/mcp",
        }
    }

    pub fn z_ai_mode(self) -> &'static str {
        match self {
            Self::Zai => "ZAI",
            Self::Zhipu => "ZHIPU",
        }
    }
}

pub fn resolve_glm(
    profile: &Profile,
    platform_override: Option<GlmPlatform>,
) -> Result<ResolvedGlm> {
    let platform = platform_override
        .map(Ok)
        .or_else(|| non_empty_value(profile, "GLM_PLATFORM").map(GlmPlatform::parse))
        .unwrap_or_else(|| {
            Ok(infer_from_mode(profile)
                .or_else(|| infer_from_base_url(profile))
                .unwrap_or(GlmPlatform::Zai))
        })?;
    let auth_token = auth_token(profile, platform)?;
    let vision_model = non_empty_value(profile, "Z_AI_VISION_MODEL")
        .unwrap_or(GLM_VISION_MODEL)
        .to_owned();
    let auto_compact_window = auto_compact_window(profile)?;

    Ok(ResolvedGlm {
        platform,
        anthropic_base_url: platform.anthropic_base_url(),
        mcp_base_url: platform.mcp_base_url(),
        z_ai_mode: platform.z_ai_mode(),
        auth_token,
        vision_model,
        auto_compact_window,
    })
}

fn infer_from_mode(profile: &Profile) -> Option<GlmPlatform> {
    match non_empty_value(profile, "Z_AI_MODE") {
        Some("ZHIPU") => Some(GlmPlatform::Zhipu),
        Some("ZAI") => Some(GlmPlatform::Zai),
        _ => None,
    }
}

fn infer_from_base_url(profile: &Profile) -> Option<GlmPlatform> {
    match non_empty_value(profile, "ANTHROPIC_BASE_URL") {
        Some(value) if value.contains("open.bigmodel.cn") => Some(GlmPlatform::Zhipu),
        Some(value) if value.contains("api.z.ai") => Some(GlmPlatform::Zai),
        _ => None,
    }
}

fn auth_token(profile: &Profile, platform: GlmPlatform) -> Result<String> {
    let platform_key = match platform {
        GlmPlatform::Zai => "GLM_ZAI_API_KEY",
        GlmPlatform::Zhipu => "GLM_ZHIPU_API_KEY",
    };
    for key in [platform_key, "Z_AI_API_KEY", "ANTHROPIC_AUTH_TOKEN"] {
        if let Some(value) = non_empty_value(profile, key) {
            return Ok(value.to_owned());
        }
    }

    bail!(
        "GLM platform '{}' must define {platform_key}, Z_AI_API_KEY, or ANTHROPIC_AUTH_TOKEN",
        platform.canonical()
    )
}

fn auto_compact_window(profile: &Profile) -> Result<String> {
    let context = parse_u64(
        non_empty_value(profile, "GLM_CONTEXT_TOKENS").unwrap_or(DEFAULT_CONTEXT_TOKENS),
        "GLM_CONTEXT_TOKENS",
    )?;
    let percent = parse_u64(
        non_empty_value(profile, "GLM_AUTO_COMPACT_PERCENT")
            .unwrap_or(DEFAULT_AUTO_COMPACT_PERCENT),
        "GLM_AUTO_COMPACT_PERCENT",
    )?;
    if percent == 0 || percent > 100 {
        bail!("GLM_AUTO_COMPACT_PERCENT must be between 1 and 100");
    }

    Ok(((context * percent) / 100).to_string())
}

fn parse_u64(value: &str, key: &str) -> Result<u64> {
    value
        .parse()
        .map_err(|_| anyhow::anyhow!("{key} must be a positive integer"))
}

fn non_empty_value<'a>(profile: &'a Profile, key: &str) -> Option<&'a str> {
    profile.value(key).filter(|value| !value.is_empty())
}
