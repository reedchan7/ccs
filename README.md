# ccs

`ccs` is a small Bash tool for switching Claude Code between three modes:

- `max`: Claude subscription login
- `api`: Anthropic API key
- `glm`: Anthropic-compatible GLM endpoint

It supports:

- session-level switching
- global default switching for new `zsh` and `bash` shells
- one-off execution with a specific profile
- shared base Claude config (`settings.json`, `CLAUDE.md`, `skills`, `plugins`, `rules`) with isolated runtime state per profile

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

Initialize the profiles you actually want:

```bash
ccs init
```

Activate a profile in the current shell:

```bash
eval "$(ccs use max)"
eval "$(ccs use api)"
eval "$(ccs use glm)"
```

Set the global default for new shells:

```bash
ccs use max --global
ccs use api --global
ccs use glm --global
```

Run Claude once under a profile without changing the current shell:

```bash
ccs run max
ccs run api -- --print "hello"
ccs run glm
ccs run -p glm
```

Show current configuration:

```bash
ccs status
```

Edit a profile manually:

```bash
ccs profile edit glm
```

## Profiles

Profiles live under:

```bash
~/.config/ccs/profiles/
```

Examples:

- `~/.config/ccs/profiles/max.env`
- `~/.config/ccs/profiles/api.env`
- `~/.config/ccs/profiles/glm.env`

Each profile must define its own `CLAUDE_CONFIG_DIR`.

Typical `glm.env`:

```dotenv
CLAUDE_CONFIG_DIR=/Users/you/.config/ccs/claude/glm
CCS_SHARED_CLAUDE_DIR=/Users/you/.claude
CCS_SHARED_PATHS=CLAUDE.md,settings.json,skills,plugins,rules
ENABLE_TOOL_SEARCH=true
ANTHROPIC_BASE_URL=https://api.z.ai/api/anthropic
ANTHROPIC_AUTH_TOKEN=your-token
ANTHROPIC_DEFAULT_OPUS_MODEL=glm-5.1
ANTHROPIC_DEFAULT_SONNET_MODEL=glm-4.7
ANTHROPIC_DEFAULT_HAIKU_MODEL=glm-4.5-air
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

If an older profile already contains local files where a shared path now belongs, `ccs` will move the local copy into:

```bash
<CLAUDE_CONFIG_DIR>/.ccs-local-backup/
```

and replace it with a symlink to the shared path.

## Repository Layout

- `bin/ccs`: main CLI implementation
- `install.sh`: installer
- `tests/test_ccs.sh`: shell-based integration tests
- `tests/lib/assert.sh`: test helpers

`install.sh` alone is not enough. `bin/ccs` is the actual tool implementation.

## Development

Run tests:

```bash
bash tests/test_ccs.sh
```

Run a syntax check:

```bash
bash -n bin/ccs install.sh tests/test_ccs.sh tests/lib/assert.sh
```

## Notes

- Secrets are stored in profile `.env` files under `~/.config/ccs/profiles/`
- Profile files are written with restrictive permissions, but they are still plain text
- `projects/` is currently not shared by default
