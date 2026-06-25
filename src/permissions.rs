use std::fs;
use std::path::{Path, PathBuf};

use anyhow::Result;
use serde_json::{json, Value};

pub fn set_bypass_permissions(project_dir: &Path) -> Result<PathBuf> {
    let claude_dir = project_dir.join(".claude");
    fs::create_dir_all(&claude_dir)?;
    let file = claude_dir.join("settings.local.json");
    let mut value: Value = if file.exists() {
        serde_json::from_str(&fs::read_to_string(&file)?)?
    } else {
        json!({})
    };

    if !value.is_object() {
        value = json!({});
    }
    let object = value.as_object_mut().expect("object set above");
    let permissions = object.entry("permissions").or_insert_with(|| json!({}));
    if !permissions.is_object() {
        *permissions = json!({});
    }
    permissions
        .as_object_mut()
        .expect("object set above")
        .insert("defaultMode".into(), json!("bypassPermissions"));

    fs::write(&file, serde_json::to_string_pretty(&value)? + "\n")?;
    Ok(file)
}
