use std::path::Path;

use anyhow::{Result, bail};

const SHARED_PATHS: &str = "CLAUDE.md,settings.json,skills,plugins,rules";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Agent {
    Max,
    Api,
    Glm,
    Mimo,
    Deepseek,
    Kimi,
}

impl Agent {
    pub fn parse(value: &str) -> Result<Self> {
        match value {
            "max" => Ok(Self::Max),
            "api" => Ok(Self::Api),
            "glm" => Ok(Self::Glm),
            "mimo" => Ok(Self::Mimo),
            "deepseek" | "ds" => Ok(Self::Deepseek),
            "kimi" => Ok(Self::Kimi),
            other => bail!(
                "unknown agent '{other}'. expected max, api, glm, mimo, deepseek, ds, or kimi"
            ),
        }
    }

    pub fn canonical(self) -> &'static str {
        match self {
            Self::Max => "max",
            Self::Api => "api",
            Self::Glm => "glm",
            Self::Mimo => "mimo",
            Self::Deepseek => "deepseek",
            Self::Kimi => "kimi",
        }
    }

    pub fn all() -> &'static [Self] {
        &[
            Self::Max,
            Self::Api,
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
            Self::Max => {}
            Self::Api => {
                values.push(("ANTHROPIC_API_KEY".into(), String::new()));
            }
            Self::Glm => {
                values.extend([
                    (
                        "ANTHROPIC_BASE_URL".into(),
                        "https://api.z.ai/api/anthropic".into(),
                    ),
                    ("ANTHROPIC_AUTH_TOKEN".into(), String::new()),
                    ("ANTHROPIC_DEFAULT_OPUS_MODEL".into(), "glm-5.2[1m]".into()),
                    (
                        "ANTHROPIC_DEFAULT_SONNET_MODEL".into(),
                        "glm-5.2[1m]".into(),
                    ),
                    ("ANTHROPIC_DEFAULT_HAIKU_MODEL".into(), "glm-4.7".into()),
                    ("API_TIMEOUT_MS".into(), "3000000".into()),
                    ("CLAUDE_CODE_AUTO_COMPACT_WINDOW".into(), "1000000".into()),
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
