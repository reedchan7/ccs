use ccs::agent::Agent;

#[test]
fn ds_is_deepseek_alias() {
    let agent = Agent::parse("ds").unwrap();
    assert_eq!(agent.canonical(), "deepseek");
}

#[test]
fn deepseek_template_contains_required_model_env() {
    let home = std::path::Path::new("/tmp/home");
    let profile = Agent::parse("deepseek").unwrap().template(home);
    let keys: Vec<_> = profile.iter().map(|(key, _)| key.as_str()).collect();
    assert!(keys.contains(&"ANTHROPIC_BASE_URL"));
    assert!(keys.contains(&"ANTHROPIC_AUTH_TOKEN"));
    assert!(keys.contains(&"ANTHROPIC_DEFAULT_HAIKU_MODEL"));
    assert!(keys.contains(&"CLAUDE_CODE_SUBAGENT_MODEL"));
}

#[test]
fn unknown_agent_is_rejected() {
    let error = Agent::parse("random").unwrap_err().to_string();
    assert!(error.contains("unknown agent"));
}
