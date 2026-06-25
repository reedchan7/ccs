# ccs

`ccs` runs Claude Code with isolated provider profiles.

It is useful when you want to keep your normal Claude Code subscription login
untouched, while launching provider-backed sessions such as GLM, DeepSeek, Kimi,
MiMo, Anthropic API, or other compatible providers from one command.

Use the plain `claude` command for your local Claude subscription. Use `ccs`
for provider profiles.

## Providers

| Provider | Command | Notes |
| --- | --- | --- |
| Anthropic API | `ccs anthropic` | Uses `ANTHROPIC_API_KEY` |
| GLM / Z.AI | `ccs glm` | Includes official GLM MCP setup |
| DeepSeek | `ccs ds` or `ccs deepseek` | `ds` is a built-in alias |
| Kimi | `ccs kimi` | Uses Kimi's coding endpoint |
| MiMo | `ccs mimo` | Uses Xiaomi MiMo's Anthropic-compatible endpoint |

Each provider has its own `CLAUDE_CONFIG_DIR`, so sessions, history, tasks, and
runtime data do not bleed between providers.

## Install

One-line install:

```bash
curl -fsSL https://raw.githubusercontent.com/reedchan7/ccs/main/install.sh | bash
```

The installer downloads the latest GitHub Release for your platform when one is
available. If no release is available yet, it falls back to building from
source. It installs `ccs` to `~/.local/bin/ccs`, installs the shell hook, and
adds `~/.local/bin` to your shell PATH if needed.

Install from a local checkout:

```bash
git clone https://github.com/reedchan7/ccs.git
cd ccs
bash install.sh
```

Then reload your shell:

```bash
source ~/.zshrc
```

For Bash:

```bash
source ~/.bashrc
```

Run the installer again to upgrade to the latest release:

```bash
curl -fsSL https://raw.githubusercontent.com/reedchan7/ccs/main/install.sh | bash
```

After `ccs` is installed, you can also upgrade with:

```bash
ccs update
```

## Quick Start

DeepSeek example:

```bash
ccs setup ds
ccs profiles edit ds
ccs
```

GLM example:

```bash
export Z_AI_API_KEY=your-zai-key
ccs setup glm -r
ccs glm
```

Daily commands:

```bash
ccs                         # open Claude Code with the default provider
ccs ds                      # open Claude Code with DeepSeek
ccs kimi --print hello      # pass args through to Claude Code
ccs setup kimi              # prepare Kimi and make it the default
ccs use ds                  # switch this shell to DeepSeek
ccs use ds --global         # make DeepSeek the default provider
ccs profiles ls             # list configured profiles
ccs profiles edit glm       # edit a provider profile
ccs status                  # show default/profile state
ccs update                  # update from GitHub Releases
```

`ccs setup` and `ccs init` are aliases. Without a provider, they default to
DeepSeek.

`ccs use <provider>` changes the current shell after the shell hook is
installed. Without the hook, it prints a one-line `eval` fallback.

## GLM Full Setup

GLM has extra first-class support because Z.AI/BigModel provides coding models
plus official MCP servers.

### Overseas Z.AI

Overseas Z.AI is the default platform:

```bash
export Z_AI_API_KEY=your-zai-key
ccs setup glm -r
ccs glm
```

This writes `GLM_ZAI_API_KEY` into `~/.config/ccs/profiles/glm.env`, opens the
profile in interactive terminals, and prepares the GLM Claude Code config.

### Domestic BigModel

Use `-p zhipu` for the domestic endpoint:

```bash
export ZHIPU_API_KEY=your-zhipu-key
ccs setup glm -r -p zhipu
ccs glm -p zhipu
```

`-p` is short for `--platform`. Supported values are:

- `zai`: overseas Z.AI, `https://api.z.ai/api/anthropic`
- `zhipu`: domestic BigModel, `https://open.bigmodel.cn/api/anthropic`

### Reconfigure

Use `-r` or `--reconfigure` when keys changed or when you want to review the
generated GLM profile:

```bash
ccs setup glm -r
```

In an interactive terminal, this refreshes the profile from environment
variables and opens `~/.config/ccs/profiles/glm.env`.

Accepted key environment variables:

- Overseas: `Z_AI_API_KEY` or `GLM_ZAI_API_KEY`
- Domestic: `ZHIPU_API_KEY` or `GLM_ZHIPU_API_KEY`

### Official MCPs

`ccs setup glm` and `ccs glm` automatically write the official GLM MCP servers
into the GLM session config:

```text
~/.config/ccs/claude/glm/.claude.json
```

Configured MCPs:

| MCP | Purpose |
| --- | --- |
| `zai-mcp-server` | Vision/multimodal support via `glm-5v-turbo` |
| `web-search-prime` | Search |
| `web-reader` | Web page reading |
| `zread` | Document reading |

These MCPs are session-scoped to `ccs glm`; they are not added to your normal
global Claude Code config.

### Models and Compaction

The generated GLM profile uses:

```dotenv
ANTHROPIC_DEFAULT_OPUS_MODEL=glm-5.2[1m]
ANTHROPIC_DEFAULT_SONNET_MODEL=glm-5.2[1m]
ANTHROPIC_DEFAULT_HAIKU_MODEL=glm-4.7
Z_AI_VISION_MODEL=glm-5v-turbo
GLM_CONTEXT_TOKENS=1000000
GLM_AUTO_COMPACT_PERCENT=90
```

`ccs` derives `CLAUDE_CODE_AUTO_COMPACT_WINDOW=900000` from the context token
count and percentage.

## Profile Isolation

Profiles live under:

```bash
~/.config/ccs/profiles/
```

Claude Code runtime directories live under:

```bash
~/.config/ccs/claude/<provider>/
```

Each provider gets isolated runtime data, including sessions, history, tasks,
telemetry, cache, and downloads.

By default, `ccs` shares these paths from your normal `~/.claude` directory:

- `CLAUDE.md`
- `settings.json`
- `skills`
- `plugins`
- `rules`

Those paths are symlinked into each provider-specific `CLAUDE_CONFIG_DIR`.

If a profile already has a local file where a shared path belongs, `ccs` moves
the local copy into:

```bash
<CLAUDE_CONFIG_DIR>/.ccs-local-backup/
```

## Profile Reference

Create or edit profiles with:

```bash
ccs profiles ls
ccs profiles edit glm
ccs profiles edit ds
```

Every profile must define `CLAUDE_CONFIG_DIR`.

Minimal Anthropic profile:

```dotenv
CLAUDE_CONFIG_DIR=/Users/you/.config/ccs/claude/anthropic
CCS_SHARED_CLAUDE_DIR=/Users/you/.claude
CCS_SHARED_PATHS=CLAUDE.md,settings.json,skills,plugins,rules
ANTHROPIC_API_KEY=your-api-key
```

Typical Anthropic-compatible provider profile:

```dotenv
CLAUDE_CONFIG_DIR=/Users/you/.config/ccs/claude/deepseek
CCS_SHARED_CLAUDE_DIR=/Users/you/.claude
CCS_SHARED_PATHS=CLAUDE.md,settings.json,skills,plugins,rules
ANTHROPIC_BASE_URL=https://api.deepseek.com/anthropic
ANTHROPIC_AUTH_TOKEN=your-provider-api-key
ANTHROPIC_DEFAULT_OPUS_MODEL=deepseek-v4-pro
ANTHROPIC_DEFAULT_SONNET_MODEL=deepseek-v4-pro
ANTHROPIC_DEFAULT_HAIKU_MODEL=deepseek-v4-flash
```

Generated provider defaults:

| Provider | Base URL | Default model |
| --- | --- | --- |
| `glm` | `https://api.z.ai/api/anthropic` | `glm-5.2[1m]` |
| `deepseek` | `https://api.deepseek.com/anthropic` | `deepseek-v4-pro` |
| `kimi` | `https://api.kimi.com/coding/` | `kimi-for-coding` |
| `mimo` | `https://api.xiaomimimo.com/anthropic` | `mimo-v2.5` |

Secrets are stored in profile `.env` files under `~/.config/ccs/profiles/`.
Profile files are written with restrictive permissions, but they are still
plain text.

## Development

```bash
make fmt
make lint
make test
make build
```

Direct commands:

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

## License

[MIT](./LICENSE)
