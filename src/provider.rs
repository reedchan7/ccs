use std::path::Path;

use anyhow::{Result, bail};
use clap::ValueEnum;

use crate::glm::GlmPlatform;

const SHARED_PATHS: &str = "CLAUDE.md,settings.json,skills,plugins,rules";
pub const GLM_VISION_MODEL: &str = "glm-5v-turbo";

#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum)]
pub enum Provider {
    Anthropic,
    Glm,
    Mimo,
    #[value(alias = "ds")]
    Deepseek,
    Kimi,
}

impl Provider {
    pub fn parse(value: &str) -> Result<Self> {
        match value {
            "anthropic" => Ok(Self::Anthropic),
            "glm" => Ok(Self::Glm),
            "mimo" => Ok(Self::Mimo),
            "deepseek" | "ds" => Ok(Self::Deepseek),
            "kimi" => Ok(Self::Kimi),
            other => bail!(
                "unknown provider '{other}'. expected anthropic, glm, mimo, deepseek, ds, or kimi"
            ),
        }
    }

    pub fn canonical(self) -> &'static str {
        match self {
            Self::Anthropic => "anthropic",
            Self::Glm => "glm",
            Self::Mimo => "mimo",
            Self::Deepseek => "deepseek",
            Self::Kimi => "kimi",
        }
    }

    pub fn all() -> &'static [Self] {
        &[
            Self::Anthropic,
            Self::Glm,
            Self::Mimo,
            Self::Deepseek,
            Self::Kimi,
        ]
    }

    pub fn template(self, home: &Path) -> Vec<(String, String)> {
        let config_dir = home
            .join(".config")
            .join("ccs")
            .join("claude")
            .join(self.canonical());
        let mut values = vec![
            ("CLAUDE_CONFIG_DIR".into(), config_dir.display().to_string()),
            (
                "CCS_SHARED_CLAUDE_DIR".into(),
                home.join(".claude").display().to_string(),
            ),
            ("CCS_SHARED_PATHS".into(), SHARED_PATHS.into()),
        ];

        match self {
            Self::Anthropic => {
                values.push(("ANTHROPIC_API_KEY".into(), String::new()));
            }
            Self::Glm => {
                values.extend([
                    ("GLM_PLATFORM".into(), GlmPlatform::Zai.canonical().into()),
                    ("GLM_ZAI_API_KEY".into(), String::new()),
                    ("GLM_ZHIPU_API_KEY".into(), String::new()),
                    (
                        "ANTHROPIC_BASE_URL".into(),
                        GlmPlatform::Zai.anthropic_base_url().into(),
                    ),
                    ("ANTHROPIC_AUTH_TOKEN".into(), String::new()),
                    ("ANTHROPIC_DEFAULT_OPUS_MODEL".into(), "glm-5.2[1m]".into()),
                    (
                        "ANTHROPIC_DEFAULT_SONNET_MODEL".into(),
                        "glm-5.2[1m]".into(),
                    ),
                    ("ANTHROPIC_DEFAULT_HAIKU_MODEL".into(), "glm-4.7".into()),
                    ("API_TIMEOUT_MS".into(), "3000000".into()),
                    ("GLM_CONTEXT_TOKENS".into(), "1000000".into()),
                    ("GLM_AUTO_COMPACT_PERCENT".into(), "90".into()),
                    ("Z_AI_MODE".into(), GlmPlatform::Zai.z_ai_mode().into()),
                    ("Z_AI_VISION_MODEL".into(), GLM_VISION_MODEL.into()),
                    (
                        "CLAUDE_CODE_DISABLE_NONESSENTIAL_TRAFFIC".into(),
                        "1".into(),
                    ),
                ]);
            }
            Self::Mimo => {
                values.extend([
                    (
                        "ANTHROPIC_BASE_URL".into(),
                        "https://api.xiaomimimo.com/anthropic".into(),
                    ),
                    ("ANTHROPIC_AUTH_TOKEN".into(), String::new()),
                    (
                        "ANTHROPIC_DEFAULT_OPUS_MODEL".into(),
                        "mimo-v2.5-pro".into(),
                    ),
                    ("ANTHROPIC_DEFAULT_SONNET_MODEL".into(), "mimo-v2.5".into()),
                    ("ANTHROPIC_DEFAULT_HAIKU_MODEL".into(), "mimo-v2.5".into()),
                ]);
            }
            Self::Deepseek => {
                values.extend([
                    (
                        "ANTHROPIC_BASE_URL".into(),
                        "https://api.deepseek.com/anthropic".into(),
                    ),
                    ("ANTHROPIC_AUTH_TOKEN".into(), String::new()),
                    (
                        "ANTHROPIC_DEFAULT_OPUS_MODEL".into(),
                        "deepseek-v4-pro".into(),
                    ),
                    (
                        "ANTHROPIC_DEFAULT_SONNET_MODEL".into(),
                        "deepseek-v4-pro".into(),
                    ),
                    (
                        "ANTHROPIC_DEFAULT_HAIKU_MODEL".into(),
                        "deepseek-v4-flash".into(),
                    ),
                    (
                        "CLAUDE_CODE_SUBAGENT_MODEL".into(),
                        "deepseek-v4-flash".into(),
                    ),
                    ("CLAUDE_CODE_EFFORT_LEVEL".into(), "max".into()),
                ]);
            }
            Self::Kimi => {
                values.extend([
                    (
                        "ANTHROPIC_BASE_URL".into(),
                        "https://api.kimi.com/coding/".into(),
                    ),
                    ("ANTHROPIC_AUTH_TOKEN".into(), String::new()),
                    (
                        "ANTHROPIC_DEFAULT_OPUS_MODEL".into(),
                        "kimi-for-coding".into(),
                    ),
                    (
                        "ANTHROPIC_DEFAULT_SONNET_MODEL".into(),
                        "kimi-for-coding".into(),
                    ),
                    (
                        "ANTHROPIC_DEFAULT_HAIKU_MODEL".into(),
                        "kimi-for-coding".into(),
                    ),
                    (
                        "CLAUDE_CODE_SUBAGENT_MODEL".into(),
                        "kimi-for-coding".into(),
                    ),
                ]);
            }
        }

        values
    }
}
