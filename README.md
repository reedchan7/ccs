# ccs

`ccs` is a Rust CLI for running Claude Code with different session-level agents.
It keeps Claude runtime data isolated per agent while sharing your base Claude
configuration.

Supported built-in agents:

- `max`: Claude subscription login
- `api`: Anthropic API key
- `glm`: Anthropic-compatible GLM endpoint
- `mimo`: Xiaomi MiMo via Anthropic-compatible endpoint
- `deepseek`: DeepSeek V4 via Anthropic-compatible endpoint
- `kimi`: Kimi coding endpoint

`ds` is a built-in alias for `deepseek`.

## Install

```bash
git clone <your-repo-url>
cd ccs
bash install.sh
```

Then reload your shell:

```bash
source ~/.zshrc
```

If you mainly use Bash:

```bash
source ~/.bashrc
```

## Quick Start

```bash
ccs init
ccs profiles add ds
ccs use ds --global
ccs
```

Daily commands:

```bash
ccs                    # open Claude Code with the default agent
ccs ds                 # open Claude Code with DeepSeek
ccs kimi --print hello # pass args through to Claude Code
ccs use ds             # use DeepSeek for plain `claude` in this shell
ccs use ds --global    # use DeepSeek by default for new shells and bare `ccs`
ccs profiles ls
ccs profiles edit ds
ccs update
```

`ccs use <agent>` changes the current shell only after `ccs init` installs the
shell hook. Without the hook, `ccs use <agent>` prints the one-line fallback
you can eval manually.

## Profiles

Profiles live under:

```bash
~/.config/ccs/profiles/
```

Examples:

- `~/.config/ccs/profiles/max.env`
- `~/.config/ccs/profiles/api.env`
- `~/.config/ccs/profiles/glm.env`
- `~/.config/ccs/profiles/mimo.env`
- `~/.config/ccs/profiles/deepseek.env`
- `~/.config/ccs/profiles/kimi.env`

Each profile must define its own `CLAUDE_CONFIG_DIR`.

Typical `glm.env`:

```dotenv
CLAUDE_CONFIG_DIR=/Users/you/.config/ccs/claude/glm
CCS_SHARED_CLAUDE_DIR=/Users/you/.claude
CCS_SHARED_PATHS=CLAUDE.md,settings.json,skills,plugins,rules
ENABLE_TOOL_SEARCH=true
# International (z.ai): https://api.z.ai/api/anthropic
# Domestic (Zhipu/bigmodel): https://open.bigmodel.cn/api/anthropic
ANTHROPIC_BASE_URL=https://api.z.ai/api/anthropic
ANTHROPIC_AUTH_TOKEN=your-token
ANTHROPIC_DEFAULT_OPUS_MODEL=glm-5.2[1m]
ANTHROPIC_DEFAULT_SONNET_MODEL=glm-5.2[1m]
ANTHROPIC_DEFAULT_HAIKU_MODEL=glm-4.7
API_TIMEOUT_MS=3000000
CLAUDE_CODE_AUTO_COMPACT_WINDOW=1000000
CLAUDE_CODE_DISABLE_NONESSENTIAL_TRAFFIC=1
```

Typical `api.env`:

```dotenv
CLAUDE_CONFIG_DIR=/Users/you/.config/ccs/claude/api
CCS_SHARED_CLAUDE_DIR=/Users/you/.claude
CCS_SHARED_PATHS=CLAUDE.md,settings.json,skills,plugins,rules
ANTHROPIC_API_KEY=your-api-key
```

Typical `max.env`:

```dotenv
CLAUDE_CONFIG_DIR=/Users/you/.config/ccs/claude/max
CCS_SHARED_CLAUDE_DIR=/Users/you/.claude
CCS_SHARED_PATHS=CLAUDE.md,settings.json,skills,plugins,rules
```

Typical `mimo.env`:

```dotenv
CLAUDE_CONFIG_DIR=/Users/you/.config/ccs/claude/mimo
CCS_SHARED_CLAUDE_DIR=/Users/you/.claude
CCS_SHARED_PATHS=CLAUDE.md,settings.json,skills,plugins,rules
ANTHROPIC_BASE_URL=https://api.xiaomimimo.com/anthropic
ANTHROPIC_AUTH_TOKEN=sk-your-mimo-key
ANTHROPIC_DEFAULT_OPUS_MODEL=mimo-v2.5-pro
ANTHROPIC_DEFAULT_SONNET_MODEL=mimo-v2.5
ANTHROPIC_DEFAULT_HAIKU_MODEL=mimo-v2.5
```

Typical `deepseek.env`:

```dotenv
CLAUDE_CONFIG_DIR=/Users/you/.config/ccs/claude/deepseek
CCS_SHARED_CLAUDE_DIR=/Users/you/.claude
CCS_SHARED_PATHS=CLAUDE.md,settings.json,skills,plugins,rules
ANTHROPIC_BASE_URL=https://api.deepseek.com/anthropic
ANTHROPIC_AUTH_TOKEN=your-deepseek-api-key
ANTHROPIC_DEFAULT_OPUS_MODEL=deepseek-v4-pro
ANTHROPIC_DEFAULT_SONNET_MODEL=deepseek-v4-pro
ANTHROPIC_DEFAULT_HAIKU_MODEL=deepseek-v4-flash
CLAUDE_CODE_SUBAGENT_MODEL=deepseek-v4-flash
CLAUDE_CODE_EFFORT_LEVEL=max
```

Typical `kimi.env`:

```dotenv
CLAUDE_CONFIG_DIR=/Users/you/.config/ccs/claude/kimi
CCS_SHARED_CLAUDE_DIR=/Users/you/.claude
CCS_SHARED_PATHS=CLAUDE.md,settings.json,skills,plugins,rules
ANTHROPIC_BASE_URL=https://api.kimi.com/coding/
ANTHROPIC_AUTH_TOKEN=your-kimi-api-key
ANTHROPIC_DEFAULT_OPUS_MODEL=kimi-for-coding
ANTHROPIC_DEFAULT_SONNET_MODEL=kimi-for-coding
ANTHROPIC_DEFAULT_HAIKU_MODEL=kimi-for-coding
CLAUDE_CODE_SUBAGENT_MODEL=kimi-for-coding
```

## Shared vs Isolated Data

By default, each profile shares these paths from `~/.claude`:

- `CLAUDE.md`
- `settings.json`
- `skills`
- `plugins`
- `rules`

These are linked into the profile-specific `CLAUDE_CONFIG_DIR`.

Runtime and history data stay isolated in each profile directory, for example:

- `sessions`
- `history.jsonl`
- `todos`
- `tasks`
- `session-env`
- `telemetry`
- `metrics`
- `debug`
- `downloads`
- `cache`

If an older profile already contains local files where a shared path now
belongs, `ccs` moves the local copy into:

```bash
<CLAUDE_CONFIG_DIR>/.ccs-local-backup/
```

and replaces it with a symlink to the shared path.

## Development

```bash
make fmt
make lint
make test
make build
```

Useful direct commands:

```bash
cargo test --all
cargo clippy --all-targets --all-features -- -D warnings
```

## Release

Push a `v*` tag to build release assets for:

- Linux x86_64
- Linux aarch64
- macOS x86_64
- macOS aarch64

Users can update from the matching GitHub Release asset with:

```bash
ccs update
```

## Notes

- Secrets are stored in profile `.env` files under `~/.config/ccs/profiles/`
- Profile files are written with restrictive permissions, but they are still plain text
- `projects/` is currently not shared by default
