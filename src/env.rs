use crate::agent::Agent;
use crate::profile::Profile;

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
    "CCS_ACTIVE_PROFILE",
];

pub fn render_shell_exports(profile: &Profile, agent: Agent) -> String {
    let mut output = String::new();
    for key in KNOWN_ENV_VARS {
        output.push_str(&format!("unset {key}\n"));
    }
    for (key, value) in profile.iter() {
        if key.starts_with("CCS_") {
            continue;
        }
        output.push_str(&format!("export {key}={}\n", shell_quote(value)));
    }
    output.push_str(&format!(
        "export CCS_ACTIVE_PROFILE={}\n",
        shell_quote(agent.canonical())
    ));
    output
}

fn shell_quote(value: &str) -> String {
    format!("'{}'", value.replace('\'', "'\\''"))
}
