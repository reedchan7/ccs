use ccs::permissions::set_bypass_permissions;
use serde_json::json;
use tempfile::TempDir;

#[test]
fn creates_settings_local_json_with_bypass_permissions() {
    let project = TempDir::new().unwrap();
    let file = set_bypass_permissions(project.path()).unwrap();
    let value: serde_json::Value =
        serde_json::from_str(&std::fs::read_to_string(file).unwrap()).unwrap();
    assert_eq!(value["permissions"]["defaultMode"], "bypassPermissions");
}

#[test]
fn preserves_existing_json_fields() {
    let project = TempDir::new().unwrap();
    std::fs::create_dir_all(project.path().join(".claude")).unwrap();
    std::fs::write(
        project.path().join(".claude/settings.local.json"),
        json!({"someOtherSetting": true, "permissions": {"allow": []}}).to_string(),
    )
    .unwrap();

    let file = set_bypass_permissions(project.path()).unwrap();
    let value: serde_json::Value =
        serde_json::from_str(&std::fs::read_to_string(file).unwrap()).unwrap();
    assert_eq!(value["someOtherSetting"], true);
    assert_eq!(value["permissions"]["allow"], json!([]));
    assert_eq!(value["permissions"]["defaultMode"], "bypassPermissions");
}
