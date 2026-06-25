use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::PathBuf;

use anyhow::{Context, Result};
use serde_json::{Value, json};

use crate::glm::{GlmPlatform, resolve_glm};
use crate::profile::Profile;
use crate::provider::Provider;

pub fn ensure_provider_mcp(
    profile: &Profile,
    provider: Provider,
    platform: Option<GlmPlatform>,
) -> Result<Option<PathBuf>> {
    if provider != Provider::Glm {
        return Ok(None);
    }
    let glm = resolve_glm(profile, platform)?;

    let config_dir = PathBuf::from(
        profile
            .value("CLAUDE_CONFIG_DIR")
            .context("GLM profile must define CLAUDE_CONFIG_DIR")?,
    );
    fs::create_dir_all(&config_dir)?;

    let file = config_dir.join(".claude.json");
    let existing_config = if file.exists() {
        Some(fs::read_to_string(&file)?)
    } else {
        None
    };
    if file.is_symlink() {
        fs::remove_file(&file)?;
    }
    let mut value = if let Some(content) = existing_config {
        serde_json::from_str(&content)
            .with_context(|| format!("invalid Claude config {}", file.display()))?
    } else {
        json!({})
    };
    if !value.is_object() {
        value = json!({});
    }

    let authorization = format!("Bearer {}", glm.auth_token);

    let object = value.as_object_mut().expect("object checked above");
    let servers = object.entry("mcpServers").or_insert_with(|| json!({}));
    if !servers.is_object() {
        *servers = json!({});
    }
    let servers = servers.as_object_mut().expect("object checked above");

    servers.insert(
        "zai-mcp-server".into(),
        json!({
            "type": "stdio",
            "command": "npx",
            "args": ["-y", "@z_ai/mcp-server@latest"],
            "env": {
                "Z_AI_API_KEY": glm.auth_token,
                "Z_AI_MODE": glm.z_ai_mode,
                "Z_AI_VISION_MODEL": glm.vision_model
            }
        }),
    );
    insert_http_mcp(
        servers,
        "web-search-prime",
        &format!("{}/web_search_prime/mcp", glm.mcp_base_url),
        &authorization,
    );
    insert_http_mcp(
        servers,
        "web-reader",
        &format!("{}/web_reader/mcp", glm.mcp_base_url),
        &authorization,
    );
    insert_http_mcp(
        servers,
        "zread",
        &format!("{}/zread/mcp", glm.mcp_base_url),
        &authorization,
    );

    fs::write(&file, serde_json::to_string_pretty(&value)? + "\n")?;
    let mut permissions = fs::metadata(&file)?.permissions();
    permissions.set_mode(0o600);
    fs::set_permissions(&file, permissions)?;
    Ok(Some(file))
}

pub fn glm_mcp_file(profile: &Profile) -> Result<PathBuf> {
    Ok(PathBuf::from(
        profile
            .value("CLAUDE_CONFIG_DIR")
            .context("GLM profile must define CLAUDE_CONFIG_DIR")?,
    )
    .join(".claude.json"))
}

pub fn glm_mcp_configured(profile: &Profile) -> Result<bool> {
    let file = glm_mcp_file(profile)?;
    if !file.exists() {
        return Ok(false);
    }
    let value: Value = serde_json::from_str(&fs::read_to_string(file)?)?;
    let Some(servers) = value.get("mcpServers").and_then(Value::as_object) else {
        return Ok(false);
    };
    Ok(
        ["zai-mcp-server", "web-search-prime", "web-reader", "zread"]
            .iter()
            .all(|name| servers.contains_key(*name)),
    )
}

fn insert_http_mcp(
    servers: &mut serde_json::Map<String, Value>,
    name: &str,
    url: &str,
    authorization: &str,
) {
    servers.insert(
        name.into(),
        json!({
            "type": "http",
            "url": url,
            "headers": {
                "Authorization": authorization
            }
        }),
    );
}
