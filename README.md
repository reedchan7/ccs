# ccs

`ccs` is a Rust CLI for running Claude Code with API provider profiles. It keeps
Claude runtime data isolated per provider while sharing your base Claude
configuration.

Supported built-in providers:

- `anthropic`: Anthropic API key
- `glm`: Anthropic-compatible GLM endpoint
- `mimo`: Xiaomi MiMo via Anthropic-compatible endpoint
- `deepseek`: DeepSeek V4 via Anthropic-compatible endpoint
- `kimi`: Kimi coding endpoint

`ds` is a built-in alias for `deepseek`.

Use the plain `claude` command for your local Claude subscription login.

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
ccs setup ds
ccs profiles edit ds
ccs
```

Daily commands:

```bash
ccs                    # open Claude Code with the default provider
ccs ds                 # open Claude Code with DeepSeek
ccs kimi --print hello # pass args through to Claude Code
ccs use ds             # use DeepSeek for plain `claude` in this shell
ccs use ds --global    # use DeepSeek by default for new shells and bare `ccs`
ccs setup kimi         # install shell hook and make Kimi the default
ccs setup anthropic    # install shell hook and make Anthropic the default
ccs profiles ls
ccs profiles edit ds
ccs update
```

`ccs setup` and `ccs init` are aliases. They install the shell hook, create the
selected profile if needed, and set it as the default provider. Without a
provider, they still default to DeepSeek. Edit the generated profile once to add
your API token.

`ccs use <provider>` changes the current shell only after `ccs setup` installs
the shell hook. Without the hook, `ccs use <provider>` prints the one-line fallback
you can eval manually.

GLM with Claude Code and official Z.AI MCPs:

```bash
ccs setup glm
ccs profiles edit glm
ccs use glm
ccs glm
ccs glm -p zhipu
ccs setup glm -p zai
ccs setup glm -r
```

`ccs glm` prepares the GLM profile-scoped Claude Code config before launch. It
adds Z.AI's official vision, web search, web reader, and ZRead MCP servers to
`~/.config/ccs/claude/glm/.claude.json`, so it does not modify your regular
Claude Code session or other provider profiles.

Inside Claude Code, use `/effort max` for harder coding tasks. Image analysis
works through `zai-mcp-server`; keep screenshots or mockups in the repo and
refer to their paths, for example `describe ./screenshots/login.png`.

GLM supports two platforms: `zai` for the international Z.AI endpoint and
`zhipu` for the domestic BigModel endpoint. Use `-p` or `--platform` on
`setup`, `use`, or direct launch. Use `-r` or `--reconfigure` to refresh the
GLM profile from `Z_AI_API_KEY` and `ZHIPU_API_KEY` in your environment, then
open the profile in an interactive terminal:

```bash
ccs setup glm -r
ccs setup glm -p zhipu
ccs use glm -p zhipu
ccs glm -p zhipu --print "describe ./screenshots/login.png"
```

## Profiles

Profiles live under:

```bash
~/.config/ccs/profiles/
```

Examples:

- `~/.config/ccs/profiles/anthropic.env`
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
GLM_PLATFORM=zai
GLM_ZAI_API_KEY=your-zai-key
GLM_ZHIPU_API_KEY=your-zhipu-key
ANTHROPIC_BASE_URL=https://api.z.ai/api/anthropic
ANTHROPIC_AUTH_TOKEN=
ANTHROPIC_DEFAULT_OPUS_MODEL=glm-5.2[1m]
ANTHROPIC_DEFAULT_SONNET_MODEL=glm-5.2[1m]
ANTHROPIC_DEFAULT_HAIKU_MODEL=glm-4.7
API_TIMEOUT_MS=3000000
GLM_CONTEXT_TOKENS=1000000
GLM_AUTO_COMPACT_PERCENT=90
CLAUDE_CODE_DISABLE_NONESSENTIAL_TRAFFIC=1
Z_AI_MODE=ZAI
Z_AI_VISION_MODEL=glm-5v-turbo
```

For GLM profiles, `ccs` derives `ANTHROPIC_BASE_URL`, `ANTHROPIC_AUTH_TOKEN`,
`Z_AI_API_KEY`, `Z_AI_MODE`, and `CLAUDE_CODE_AUTO_COMPACT_WINDOW` from
`GLM_PLATFORM`, the matching `GLM_*_API_KEY`, `GLM_CONTEXT_TOKENS`, and
`GLM_AUTO_COMPACT_PERCENT`. It also writes the matching MCP auth headers and
platform endpoints into the GLM profile's own Claude config before starting
Claude Code.

Typical `anthropic.env`:

```dotenv
CLAUDE_CONFIG_DIR=/Users/you/.config/ccs/claude/anthropic
CCS_SHARED_CLAUDE_DIR=/Users/you/.claude
CCS_SHARED_PATHS=CLAUDE.md,settings.json,skills,plugins,rules
ANTHROPIC_API_KEY=your-api-key
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
