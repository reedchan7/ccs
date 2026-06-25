# ccs Rust 2024 Redesign Spec

## Summary

Rewrite `ccs` as a Rust Edition 2024 CLI for Claude Code session-level agent
selection. The top-level command remains `ccs`. The daily path becomes:

```bash
ccs                 # open Claude Code with the default agent
ccs ds              # open Claude Code with DeepSeek
ccs kimi --print hi # run Claude Code once with Kimi and pass args through
ccs use ds          # make the current shell use DeepSeek for plain `claude`
ccs use ds --global # make new shells use DeepSeek by default
```

The design favors short commands for daily work and grouped commands for
configuration management:

```bash
ccs profiles
ccs profiles ls
ccs profiles edit ds
ccs init
ccs update
ccs permissions bypass
```

## Goals

- Keep `ccs` as the installed executable name.
- Make `ccs` with no arguments enter the default Claude Code agent.
- Make `ccs <agent>` enter a specific Claude Code agent directly.
- Support `ds` as a stable alias for `deepseek` everywhere an agent name is
  accepted.
- Rebuild the implementation in Rust Edition 2024.
- Preserve the existing goal: session-level Claude Code use with multiple
  Anthropic-compatible models.
- Keep the existing on-disk profile data usable under `~/.config/ccs`.
- Add self-update from GitHub Releases.
- Add formatting, linting, Makefile targets, CI, and release builds for:
  - Linux x86_64
  - Linux aarch64
  - macOS x86_64
  - macOS aarch64

## Non-Goals

- No TUI.
- No daemon.
- No account sync service.
- No new plugin system.
- No attempt to manage Claude Code installation itself.
- No migration away from plain profile files unless the Rust implementation can
  read the current `.env` files cleanly first.

## Research Notes

The command model is based on common CLI conventions from tools that switch
runtime context:

- `conda activate <env>` makes environment activation explicit.
- `nvm use <version>` and `mise use <tool>` make "use this now" a familiar
  action.
- `mise use --global` keeps the action and changes the scope with a flag.
- `docker context ls/use` and `kubectl config use-context` keep context objects
  grouped under nouns.
- CLI Guidelines and Microsoft command-line guidance favor short lowercase
  commands, stable aliases, and clear command grouping.

References:

- https://clig.dev/
- https://learn.microsoft.com/en-us/dotnet/standard/commandline/design-guidance
- https://mise.jdx.dev/cli/use.html
- https://github.com/pyenv/pyenv/blob/master/COMMANDS.md
- https://docs.conda.io/projects/conda/en/latest/user-guide/tasks/manage-environments.html
- https://docs.docker.com/reference/cli/docker/context/use/
- https://kubernetes.io/docs/reference/kubectl/generated/kubectl_config/kubectl_config_use-context/

## Command Design

### Daily Agent Entry

```bash
ccs
ccs <agent> [claude args...]
```

Behavior:

- `ccs` resolves the global default agent and `exec`s `claude`.
- `ccs <agent>` resolves the agent, applies its environment, and `exec`s
  `claude`.
- Any extra arguments are passed to Claude Code unchanged.
- `ccs ds` is equivalent to `ccs deepseek`.
- If no default agent exists, `ccs` prints a short next-step message:

```text
No default agent set.
Run: ccs use ds --global
See: ccs profiles
```

Examples:

```bash
ccs
ccs ds
ccs kimi --print "hello"
ccs glm --continue
ccs api --resume
```

### Current Shell Selection

```bash
ccs use <agent>
ccs use <agent> --global
```

Behavior:

- With the shell hook installed, `ccs use <agent>` updates the current shell
  environment.
- Without the shell hook, it prints the eval fallback and a short `ccs init`
  hint.
- `--global` writes the default agent used by new shells and by bare `ccs`.
- `ccs use ds --global` writes `deepseek` as the canonical stored value.

The shell hook is allowed to define a shell function named `ccs` that calls the
Rust binary for all commands and evaluates a machine-readable env payload only
for `ccs use`.

### Profile Management

```bash
ccs profiles
ccs profiles list
ccs profiles ls
ccs profiles edit <agent>
ccs profiles add <agent>
ccs profiles remove <agent>
```

Behavior:

- `ccs profiles`, `ccs profiles list`, and `ccs profiles ls` list configured
  profiles.
- `ccs profiles edit <agent>` opens the profile file in `$EDITOR`.
- `ccs profiles add <agent>` creates a stub or guided profile.
- `ccs profiles remove <agent>` removes only the profile `.env` file after an
  interactive confirmation unless `--yes` is supplied.
- Secrets are never printed.

### Setup, Status, and Maintenance

```bash
ccs init
ccs status
ccs update
ccs permissions bypass
```

Behavior:

- `ccs init` runs first-time setup:
  - creates `~/.config/ccs`
  - offers guided profile creation
  - installs shell hook for zsh/bash/fish where supported
- `ccs status` shows:
  - active session agent
  - global default agent
  - configured profiles
  - profile config directories
  - shell hook status
- `ccs update` self-updates from the latest GitHub Release asset matching the
  current OS and CPU architecture.
- `ccs permissions bypass` writes Claude Code local permissions for the current
  project by setting `permissions.defaultMode` to `bypassPermissions` in
  `.claude/settings.local.json`.

## Agent Names

Built-in agent names:

- `max`
- `api`
- `glm`
- `mimo`
- `deepseek`
- `kimi`

Built-in aliases:

- `ds` -> `deepseek`

Rules:

- Alias normalization happens before file lookup, env application, status
  output, and global default writes.
- Status output uses canonical names.
- Unknown agent names are accepted only through explicit `profiles add <name>`
  if the design later allows custom profiles. The first Rust version can keep
  the current fixed set.

## Configuration and State

Continue using:

```text
~/.config/ccs/
  config
  ccs.sh
  profiles/
    max.env
    api.env
    glm.env
    mimo.env
    deepseek.env
    kimi.env
  claude/
    max/
    api/
    glm/
    mimo/
    deepseek/
    kimi/
```

Profile files remain dotenv-style:

```dotenv
CLAUDE_CONFIG_DIR=/Users/you/.config/ccs/claude/deepseek
CCS_SHARED_CLAUDE_DIR=/Users/you/.claude
CCS_SHARED_PATHS=CLAUDE.md,settings.json,skills,plugins,rules
ANTHROPIC_BASE_URL=https://api.deepseek.com/anthropic
ANTHROPIC_AUTH_TOKEN=your-token
ANTHROPIC_DEFAULT_OPUS_MODEL=deepseek-v4-pro
ANTHROPIC_DEFAULT_SONNET_MODEL=deepseek-v4-pro
ANTHROPIC_DEFAULT_HAIKU_MODEL=deepseek-v4-flash
CLAUDE_CODE_SUBAGENT_MODEL=deepseek-v4-flash
CLAUDE_CODE_EFFORT_LEVEL=max
```

The existing `config` file can remain key-value:

```text
default_profile=deepseek
```

Rust can add a TOML config later only if key-value config becomes a real
limitation.

## Environment Semantics

When activating or running an agent:

1. Load the profile file.
2. Require `CLAUDE_CONFIG_DIR`.
3. Require credentials and base URL fields based on agent type:
   - `api`: `ANTHROPIC_API_KEY`
   - `glm`, `mimo`, `deepseek`, `kimi`: `ANTHROPIC_BASE_URL`,
     `ANTHROPIC_AUTH_TOKEN`
   - `max`: no API secret required
4. Ensure shared Claude config paths are symlinked into `CLAUDE_CONFIG_DIR`.
5. Clear known Claude/Anthropic/ccs env vars that could leak across agents.
6. Export all non-`CCS_` profile keys.
7. Set `CCS_ACTIVE_PROFILE=<canonical-agent>`.

Known env vars to clear:

```text
CLAUDE_CONFIG_DIR
CCS_SHARED_CLAUDE_DIR
CCS_SHARED_PATHS
ANTHROPIC_API_KEY
ANTHROPIC_AUTH_TOKEN
ANTHROPIC_BASE_URL
ANTHROPIC_MODEL
ANTHROPIC_DEFAULT_OPUS_MODEL
ANTHROPIC_DEFAULT_SONNET_MODEL
ANTHROPIC_DEFAULT_HAIKU_MODEL
ENABLE_TOOL_SEARCH
CLAUDE_CODE_DISABLE_EXPERIMENTAL_BETAS
CLAUDE_CODE_SUBAGENT_MODEL
CLAUDE_CODE_EFFORT_LEVEL
API_TIMEOUT_MS
CLAUDE_CODE_AUTO_COMPACT_WINDOW
CLAUDE_CODE_DISABLE_NONESSENTIAL_TRAFFIC
CCS_ACTIVE_PROFILE
```

## Shared Claude Config

Default shared paths:

```text
CLAUDE.md,settings.json,skills,plugins,rules
```

Behavior:

- Shared source defaults to `~/.claude`.
- Shared paths are configured by `CCS_SHARED_PATHS`.
- Missing shared source paths are skipped.
- Existing local target paths are moved to
  `<CLAUDE_CONFIG_DIR>/.ccs-local-backup/` before the symlink is created.
- Existing correct symlinks are left as-is.

Runtime and history data stay isolated per profile:

```text
sessions
history.jsonl
todos
tasks
session-env
telemetry
metrics
debug
downloads
cache
```

## Rust Architecture

Proposed crate layout:

```text
src/
  main.rs        # CLI parse and command dispatch
  cli.rs         # clap command model
  agent.rs       # agent canonicalization, aliases, built-in templates
  config.rs      # config paths, profile file read/write
  env.rs         # env payload creation and process execution
  links.rs       # shared Claude config symlink handling
  shell.rs       # shell hook generation and install
  update.rs      # GitHub Release self-update
  permissions.rs # .claude/settings.local.json mutation
```

Recommended crates:

- `clap` with derive support for CLI parsing.
- `anyhow` for command-level error context.
- `thiserror` only if library-style typed errors become useful.
- `serde` and `serde_json` for Claude permissions JSON.
- `directories` for config directory resolution.
- `dotenvy` or a small parser for profile files. Prefer `dotenvy` if it
  preserves current profile behavior well enough.
- `self_update` for GitHub Release updates if it supports the release asset
  naming cleanly; otherwise use `ureq` plus atomic replace.
- `assert_cmd`, `predicates`, and `tempfile` for CLI tests.

Rust edition:

```toml
edition = "2024"
```

## Shell Hook Design

The installed hook should be small and boring. For zsh/bash:

```bash
ccs() {
  local command="${1:-}"
  case " $* " in
    *" --global "*) "$HOME/.local/bin/ccs" "$@" ;;
    *) if [ "$command" = "use" ]; then
         local output
         output="$("$HOME/.local/bin/ccs" internal env "$@")" || {
           printf '%s\n' "$output" >&2
           return 1
         }
         eval "$output"
       else
         "$HOME/.local/bin/ccs" "$@"
       fi ;;
  esac
}
```

The Rust binary owns all logic. The shell function only bridges the parent-shell
environment limitation for `ccs use`.

## Install Design

The repository should produce a single binary named `ccs`.

Install paths:

```text
~/.local/bin/ccs
~/.config/ccs/ccs.sh
```

`install.sh` can remain as a thin bootstrapper that downloads or copies the
compiled binary and runs:

```bash
ccs init --hooks-only
```

The Makefile should include:

```makefile
fmt
lint
test
build
install
release-local
```

## CI and Release

CI on pull requests and pushes:

- `cargo fmt --all -- --check`
- `cargo clippy --all-targets --all-features -- -D warnings`
- `cargo test --all`
- `cargo build --locked`

Release on tags matching `v*`:

- Build binaries for:
  - `x86_64-unknown-linux-gnu`
  - `aarch64-unknown-linux-gnu`
  - `x86_64-apple-darwin`
  - `aarch64-apple-darwin`
- Package assets as:
  - `ccs-vX.Y.Z-x86_64-unknown-linux-gnu.tar.gz`
  - `ccs-vX.Y.Z-aarch64-unknown-linux-gnu.tar.gz`
  - `ccs-vX.Y.Z-x86_64-apple-darwin.tar.gz`
  - `ccs-vX.Y.Z-aarch64-apple-darwin.tar.gz`
- Publish a GitHub Release with checksums.

Use `cargo dist` if it keeps the release workflow simpler and still produces the
required assets. If it adds friction, use direct GitHub Actions matrix builds.

## Test Coverage

The Rust rewrite should cover these behaviors:

- `ccs` without a default prints the short next-step message.
- `ccs` with a default executes `claude` with that profile env.
- `ccs ds` resolves to `deepseek`.
- `ccs ds --print hi` passes args through unchanged.
- `ccs use ds` emits shell env for the canonical profile.
- `ccs use ds --global` writes `default_profile=deepseek`.
- `ccs profiles`, `ccs profiles ls`, and `ccs profiles list` list profiles.
- `ccs profiles edit ds` opens the canonical profile file.
- `ccs permissions bypass` preserves unrelated JSON and sets
  `permissions.defaultMode`.
- Shared Claude config symlinks are created.
- Conflicting local shared paths are moved to `.ccs-local-backup`.
- Secrets do not appear in status/profile listing output.

## Migration Plan

1. Add Rust project files while leaving the current Bash implementation in
   place.
2. Implement profile parsing and env generation first.
3. Implement direct agent entry with fake `claude` tests.
4. Implement `use`, global default, shell hook, and init.
5. Implement profile management and permissions command.
6. Replace `bin/ccs` with a wrapper or move the Rust binary install target to
   `ccs`.
7. Update README to document the new daily workflow.
8. Add CI and release workflows.
9. Add self-update once release asset names are stable.

## Acceptance Criteria

- `cargo fmt`, `cargo clippy`, `cargo test`, and release build commands pass.
- `make fmt`, `make lint`, `make test`, and `make build` work locally.
- `ccs` enters the default Claude Code agent.
- `ccs ds` enters DeepSeek.
- `ccs kimi --print hello` passes Claude args unchanged.
- `ccs use ds` works in an initialized shell.
- `ccs use ds --global` sets the default for future bare `ccs`.
- Existing `.env` profile files under `~/.config/ccs/profiles` keep working.
- Linux/macOS AMD64/ARM64 release assets are built from version tags.
- `ccs update` installs the matching GitHub Release asset for the current
  platform.
