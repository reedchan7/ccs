use std::collections::BTreeMap;
use std::fs::{self, File};
use std::io::Write;
use std::os::unix::fs::PermissionsExt;
use std::path::PathBuf;

use anyhow::{Context, Result, bail};

use crate::glm::GlmPlatform;
use crate::paths::Paths;
use crate::provider::Provider;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Profile {
    values: BTreeMap<String, String>,
}

impl Profile {
    pub fn load(paths: &Paths, provider: Provider) -> Result<Self> {
        let file = paths.profile_file(provider);
        let iter = dotenvy::from_path_iter(&file)
            .with_context(|| format!("profile '{}' is not configured", provider.canonical()))?;
        let mut values = BTreeMap::new();
        for item in iter {
            let (key, value) =
                item.with_context(|| format!("invalid profile file {}", file.display()))?;
            values.insert(key, value);
        }
        let profile = Self { values };
        profile.validate(provider)?;
        Ok(profile)
    }

    pub fn value(&self, key: &str) -> Option<&str> {
        self.values.get(key).map(String::as_str)
    }

    pub fn iter(&self) -> impl Iterator<Item = (&String, &String)> {
        self.values.iter()
    }

    fn validate(&self, provider: Provider) -> Result<()> {
        self.require(provider, "CLAUDE_CONFIG_DIR")?;
        match provider {
            Provider::Anthropic => self.require(provider, "ANTHROPIC_API_KEY")?,
            Provider::Glm => self.require_any(
                provider,
                &[
                    "GLM_ZAI_API_KEY",
                    "GLM_ZHIPU_API_KEY",
                    "ZAI_API_KEY",
                    "Z_AI_API_KEY",
                    "ZHIPU_API_KEY",
                    "ANTHROPIC_AUTH_TOKEN",
                ],
            )?,
            Provider::Mimo | Provider::Deepseek | Provider::Kimi => {
                self.require(provider, "ANTHROPIC_BASE_URL")?;
                self.require(provider, "ANTHROPIC_AUTH_TOKEN")?;
            }
        }
        Ok(())
    }

    fn require(&self, provider: Provider, key: &str) -> Result<()> {
        match self.value(key) {
            Some(value) if !value.is_empty() => Ok(()),
            _ => bail!("profile '{}' must define {key}", provider.canonical()),
        }
    }

    fn require_any(&self, provider: Provider, keys: &[&str]) -> Result<()> {
        if keys.iter().any(|key| {
            self.value(key)
                .map(|value| !value.is_empty())
                .unwrap_or(false)
        }) {
            return Ok(());
        }

        bail!(
            "profile '{}' must define one of {}",
            provider.canonical(),
            keys.join(", ")
        )
    }
}

pub fn write_template(paths: &Paths, provider: Provider) -> Result<PathBuf> {
    fs::create_dir_all(paths.profiles_dir())?;
    let file = paths.profile_file(provider);
    if file.exists() {
        return Ok(file);
    }
    let mut handle = File::create(&file)?;
    for (key, value) in provider.template(paths.home()) {
        writeln!(handle, "{key}={value}")?;
    }
    let mut permissions = handle.metadata()?.permissions();
    permissions.set_mode(0o600);
    fs::set_permissions(&file, permissions)?;
    Ok(file)
}

pub fn write_glm_defaults(
    paths: &Paths,
    platform_override: Option<GlmPlatform>,
    reconfigure: bool,
) -> Result<PathBuf> {
    let file = write_template(paths, Provider::Glm)?;
    let existing = read_profile_values(&file)?;
    let platform = platform_override
        .or_else(|| {
            existing
                .get("GLM_PLATFORM")
                .and_then(|value| GlmPlatform::parse(value).ok())
        })
        .or_else(|| match existing.get("Z_AI_MODE").map(String::as_str) {
            Some("ZHIPU") => Some(GlmPlatform::Zhipu),
            Some("ZAI") => Some(GlmPlatform::Zai),
            _ => None,
        })
        .or_else(|| match existing.get("ANTHROPIC_BASE_URL") {
            Some(value) if value.contains("open.bigmodel.cn") => Some(GlmPlatform::Zhipu),
            Some(value) if value.contains("api.z.ai") => Some(GlmPlatform::Zai),
            _ => None,
        })
        .unwrap_or(GlmPlatform::Zai);
    let active_token = active_glm_token(&existing, platform, reconfigure);
    let zai_token = keyed_token(
        existing.get("GLM_ZAI_API_KEY"),
        first_env(&["GLM_ZAI_API_KEY", "ZAI_API_KEY", "Z_AI_API_KEY"]),
        reconfigure,
    );
    let zhipu_token = keyed_token(
        existing.get("GLM_ZHIPU_API_KEY"),
        first_env(&["GLM_ZHIPU_API_KEY", "ZHIPU_API_KEY"]),
        reconfigure,
    );

    let mut updates = Vec::new();
    for (key, template_value) in Provider::Glm.template(paths.home()) {
        let value = match key.as_str() {
            "GLM_PLATFORM" => platform.canonical().into(),
            "ANTHROPIC_BASE_URL" => platform.anthropic_base_url().into(),
            "Z_AI_MODE" => platform.z_ai_mode().into(),
            "GLM_ZAI_API_KEY" if platform == GlmPlatform::Zai => {
                active_token.clone().unwrap_or_default()
            }
            "GLM_ZHIPU_API_KEY" if platform == GlmPlatform::Zhipu => {
                active_token.clone().unwrap_or_default()
            }
            "GLM_ZAI_API_KEY" => zai_token.clone().unwrap_or_default(),
            "GLM_ZHIPU_API_KEY" => zhipu_token.clone().unwrap_or_default(),
            "ANTHROPIC_AUTH_TOKEN" => active_token
                .clone()
                .or_else(|| existing_non_empty(&existing, &key))
                .unwrap_or_default(),
            "CLAUDE_CONFIG_DIR"
            | "CCS_SHARED_CLAUDE_DIR"
            | "CCS_SHARED_PATHS"
            | "GLM_CONTEXT_TOKENS" => existing.get(&key).cloned().unwrap_or(template_value),
            "GLM_AUTO_COMPACT_PERCENT" => match existing.get(&key).map(String::as_str) {
                Some("85") | None => template_value,
                Some(value) => value.to_owned(),
            },
            _ => template_value,
        };
        updates.push((key, value));
    }

    write_profile_values(paths, Provider::Glm, &updates)
}

fn active_glm_token(
    existing: &BTreeMap<String, String>,
    platform: GlmPlatform,
    reconfigure: bool,
) -> Option<String> {
    let existing_token = match platform {
        GlmPlatform::Zai => existing_non_empty(existing, "GLM_ZAI_API_KEY"),
        GlmPlatform::Zhipu => existing_non_empty(existing, "GLM_ZHIPU_API_KEY"),
    }
    .or_else(|| {
        if platform == GlmPlatform::Zhipu {
            existing_non_empty(existing, "ZHIPU_API_KEY")
        } else {
            None
        }
    })
    .or_else(|| existing_non_empty(existing, "ZAI_API_KEY"))
    .or_else(|| existing_non_empty(existing, "Z_AI_API_KEY"))
    .or_else(|| existing_non_empty(existing, "ANTHROPIC_AUTH_TOKEN"));

    let env_token = match platform {
        GlmPlatform::Zai => first_env(&["GLM_ZAI_API_KEY", "ZAI_API_KEY", "Z_AI_API_KEY"]),
        GlmPlatform::Zhipu => first_env(&[
            "GLM_ZHIPU_API_KEY",
            "ZHIPU_API_KEY",
            "ZAI_API_KEY",
            "Z_AI_API_KEY",
        ]),
    }
    .or_else(|| first_env(&["ANTHROPIC_AUTH_TOKEN"]));

    if reconfigure {
        env_token.or(existing_token)
    } else {
        existing_token.or(env_token)
    }
}

fn keyed_token(
    existing: Option<&String>,
    env_value: Option<String>,
    reconfigure: bool,
) -> Option<String> {
    let existing = existing.filter(|value| !value.is_empty()).cloned();
    if reconfigure {
        env_value.or(existing)
    } else {
        existing.or(env_value)
    }
}

fn existing_non_empty(existing: &BTreeMap<String, String>, key: &str) -> Option<String> {
    existing.get(key).filter(|value| !value.is_empty()).cloned()
}

fn first_env(keys: &[&str]) -> Option<String> {
    keys.iter()
        .filter_map(|key| std::env::var(key).ok())
        .find(|value| !value.is_empty())
}

fn write_profile_values(
    paths: &Paths,
    provider: Provider,
    values: &[(String, String)],
) -> Result<PathBuf> {
    let file = write_template(paths, provider)?;
    let content = fs::read_to_string(&file).unwrap_or_default();
    let mut next = String::new();
    let mut seen = vec![false; values.len()];

    for line in content.lines() {
        let mut replaced = false;
        for (index, (key, value)) in values.iter().enumerate() {
            if line.starts_with(&format!("{key}=")) {
                next.push_str(&format!("{key}={value}\n"));
                seen[index] = true;
                replaced = true;
                break;
            }
        }
        if !replaced {
            next.push_str(line);
            next.push('\n');
        }
    }

    for (index, (key, value)) in values.iter().enumerate() {
        if !seen[index] {
            next.push_str(&format!("{key}={value}\n"));
        }
    }

    fs::write(&file, next)?;
    let mut permissions = fs::metadata(&file)?.permissions();
    permissions.set_mode(0o600);
    fs::set_permissions(&file, permissions)?;
    Ok(file)
}

fn read_profile_values(file: &PathBuf) -> Result<BTreeMap<String, String>> {
    if !file.exists() {
        return Ok(BTreeMap::new());
    }
    let mut values = BTreeMap::new();
    for item in dotenvy::from_path_iter(file)? {
        let (key, value) = item?;
        values.insert(key, value);
    }
    Ok(values)
}

pub fn read_default_profile(paths: &Paths) -> Result<Option<Provider>> {
    let file = paths.config_file();
    if !file.exists() {
        return Ok(None);
    }
    let content = fs::read_to_string(file)?;
    for line in content.lines() {
        if let Some(value) = line.strip_prefix("default_profile=") {
            return Ok(Some(Provider::parse(value)?));
        }
    }
    Ok(None)
}

pub fn write_default_profile(paths: &Paths, provider: Provider) -> Result<()> {
    fs::create_dir_all(paths.ccs_home())?;
    fs::write(
        paths.config_file(),
        format!("default_profile={}\n", provider.canonical()),
    )?;
    Ok(())
}
