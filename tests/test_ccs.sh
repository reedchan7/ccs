#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
ORIG_PATH="${PATH}"
# shellcheck disable=SC1091
source "${ROOT_DIR}/tests/lib/assert.sh"

failures=0

setup_test_env() {
  TEST_HOME="$(mktemp -d)"
  TEST_BIN="${TEST_HOME}/bin"
  mkdir -p "${TEST_BIN}"
  cat >"${TEST_BIN}/claude" <<'EOF'
#!/usr/bin/env bash
printf 'CCS_ACTIVE_PROFILE=%s\n' "${CCS_ACTIVE_PROFILE:-}"
printf 'CLAUDE_CONFIG_DIR=%s\n' "${CLAUDE_CONFIG_DIR:-}"
printf 'ANTHROPIC_API_KEY=%s\n' "${ANTHROPIC_API_KEY:-}"
printf 'ANTHROPIC_AUTH_TOKEN=%s\n' "${ANTHROPIC_AUTH_TOKEN:-}"
printf 'ANTHROPIC_BASE_URL=%s\n' "${ANTHROPIC_BASE_URL:-}"
printf 'ANTHROPIC_DEFAULT_OPUS_MODEL=%s\n' "${ANTHROPIC_DEFAULT_OPUS_MODEL:-}"
printf 'ANTHROPIC_DEFAULT_SONNET_MODEL=%s\n' "${ANTHROPIC_DEFAULT_SONNET_MODEL:-}"
printf 'ANTHROPIC_DEFAULT_HAIKU_MODEL=%s\n' "${ANTHROPIC_DEFAULT_HAIKU_MODEL:-}"
printf 'ARGS='
printf '%s ' "$@"
printf '\n'
EOF
  chmod +x "${TEST_BIN}/claude"
}

cleanup_test_env() {
  rm -rf "${TEST_HOME}"
}

setup_shared_claude_base() {
  mkdir -p "${TEST_HOME}/.claude/skills" "${TEST_HOME}/.claude/plugins" "${TEST_HOME}/.claude/rules"
  printf 'base settings\n' > "${TEST_HOME}/.claude/settings.json"
  printf 'base claude md\n' > "${TEST_HOME}/.claude/CLAUDE.md"
  printf 'skill data\n' > "${TEST_HOME}/.claude/skills/example.txt"
  printf 'plugin data\n' > "${TEST_HOME}/.claude/plugins/example.txt"
  printf 'rule data\n' > "${TEST_HOME}/.claude/rules/example.txt"
}

write_profile() {
  local profile_name="$1"
  shift
  local profile_dir="${TEST_HOME}/.config/ccs/profiles"
  mkdir -p "${profile_dir}"
  printf '%s\n' "$@" >"${profile_dir}/${profile_name}.env"
}

write_config() {
  mkdir -p "${TEST_HOME}/.config/ccs"
  printf '%s\n' "$@" >"${TEST_HOME}/.config/ccs/config"
}

run_install() {
  TEST_OUTPUT="$(
    HOME="${TEST_HOME}" \
    PATH="${TEST_BIN}:${ORIG_PATH}" \
    bash "${ROOT_DIR}/install.sh" 2>&1
  )"
  TEST_STATUS=$?
}

run_ccs_with_input() {
  local input="$1"
  shift
  TEST_OUTPUT="$(
    printf '%s' "${input}" | \
      HOME="${TEST_HOME}" \
      PATH="${TEST_BIN}:${ORIG_PATH}" \
      "${ROOT_DIR}/bin/ccs" "$@" 2>&1
  )"
  TEST_STATUS=$?
}

test_help_shows_main_commands() {
  run_ccs help
  assert_status 0
  assert_output_contains "use <profile>"
  assert_output_contains "run [profile|-p profile]"
}

test_use_emits_exports_for_api_profile() {
  write_profile "api" \
    "CLAUDE_CONFIG_DIR=${TEST_HOME}/.config/ccs/claude/api" \
    "ANTHROPIC_API_KEY=test-key"

  run_ccs use api

  assert_status 0
  assert_output_contains "unset ANTHROPIC_AUTH_TOKEN"
  assert_output_contains "export ANTHROPIC_API_KEY=test-key"
  assert_output_contains "export CCS_ACTIVE_PROFILE=api"
}

test_run_uses_explicit_profile_and_invokes_claude() {
  write_profile "glm" \
    "CLAUDE_CONFIG_DIR=${TEST_HOME}/.config/ccs/claude/glm" \
    "ANTHROPIC_BASE_URL=https://example.test" \
    "ANTHROPIC_AUTH_TOKEN=glm-token" \
    "ANTHROPIC_DEFAULT_OPUS_MODEL=glm-5.1"

  run_ccs run glm -- --print hello

  assert_status 0
  assert_output_contains "CCS_ACTIVE_PROFILE=glm"
  assert_output_contains "CLAUDE_CONFIG_DIR=${TEST_HOME}/.config/ccs/claude/glm"
  assert_output_contains "ANTHROPIC_BASE_URL=https://example.test"
  assert_output_contains "ANTHROPIC_AUTH_TOKEN=glm-token"
  assert_output_contains "ANTHROPIC_DEFAULT_OPUS_MODEL=glm-5.1"
  assert_output_contains "ARGS=--print hello "
}

test_use_global_sets_default_profile() {
  write_profile "max" \
    "CLAUDE_CONFIG_DIR=${TEST_HOME}/.config/ccs/claude/max"

  run_ccs use max --global

  assert_status 0
  assert_file_contains "${TEST_HOME}/.config/ccs/config" "default_profile=max"
}

test_run_without_profile_uses_global_default() {
  write_profile "max" \
    "CLAUDE_CONFIG_DIR=${TEST_HOME}/.config/ccs/claude/max"
  write_config "default_profile=max"

  run_ccs run -- --print hi

  assert_status 0
  assert_output_contains "CCS_ACTIVE_PROFILE=max"
  assert_output_contains "CLAUDE_CONFIG_DIR=${TEST_HOME}/.config/ccs/claude/max"
  assert_output_contains "ARGS=--print hi "
}

test_status_reports_profiles_without_secrets() {
  write_profile "api" \
    "CLAUDE_CONFIG_DIR=${TEST_HOME}/.config/ccs/claude/api" \
    "ANTHROPIC_API_KEY=super-secret"
  write_config "default_profile=api"

  run_ccs status

  assert_status 0
  assert_output_contains "Global profile: api"
  assert_output_contains "Configured profiles:"
  assert_output_contains "api -> ${TEST_HOME}/.config/ccs/claude/api"
  assert_output_not_contains "super-secret"
}

test_install_writes_ccs_binary_and_shell_hooks() {
  run_install

  assert_status 0
  assert_file_exists "${TEST_HOME}/.local/bin/ccs"
  assert_file_exists "${TEST_HOME}/.config/ccs/ccs.sh"
  assert_file_contains "${TEST_HOME}/.zshrc" "ccs shell hook"
  assert_file_contains "${TEST_HOME}/.bashrc" "ccs shell hook"

  run_install

  assert_status 0
  assert_equals "1" "$(grep -c '# >>> ccs shell hook >>>' "${TEST_HOME}/.zshrc")"
  assert_equals "1" "$(grep -c '# >>> ccs shell hook >>>' "${TEST_HOME}/.bashrc")"
}

test_installed_hook_applies_global_profile_to_new_shell() {
  write_profile "max" \
    "CLAUDE_CONFIG_DIR=${TEST_HOME}/.config/ccs/claude/max"
  write_config "default_profile=max"

  run_install

  assert_status 0

  TEST_OUTPUT="$(
    HOME="${TEST_HOME}" \
    bash -c 'source "$HOME/.config/ccs/ccs.sh"; printf "CCS_ACTIVE_PROFILE=%s\n" "${CCS_ACTIVE_PROFILE:-}"; printf "CLAUDE_CONFIG_DIR=%s\n" "${CLAUDE_CONFIG_DIR:-}"'
  )"
  TEST_STATUS=$?

  assert_status 0
  assert_output_contains "CCS_ACTIVE_PROFILE=max"
  assert_output_contains "CLAUDE_CONFIG_DIR=${TEST_HOME}/.config/ccs/claude/max"
}

test_init_creates_only_selected_profiles() {
  run_ccs_with_input $'y\n\nn\nn\nn\n' init

  assert_status 0
  assert_file_exists "${TEST_HOME}/.config/ccs/profiles/max.env"
  assert_file_not_exists "${TEST_HOME}/.config/ccs/profiles/api.env"
  assert_file_not_exists "${TEST_HOME}/.config/ccs/profiles/glm.env"
  assert_file_contains "${TEST_HOME}/.config/ccs/profiles/max.env" "CLAUDE_CONFIG_DIR=${TEST_HOME}/.config/ccs/claude/max"
  assert_file_contains "${TEST_HOME}/.config/ccs/profiles/max.env" "CCS_SHARED_CLAUDE_DIR=${TEST_HOME}/.claude"
  assert_file_contains "${TEST_HOME}/.config/ccs/profiles/max.env" "CCS_SHARED_PATHS=CLAUDE.md,settings.json,skills,plugins,rules"
  assert_output_contains 'To activate in the current shell: eval "$(ccs use <profile>)"'
}

test_profile_edit_creates_stub_and_invokes_editor() {
  cat >"${TEST_BIN}/fake-editor" <<'EOF'
#!/usr/bin/env bash
printf '# edited by test\n' >>"$1"
EOF
  chmod +x "${TEST_BIN}/fake-editor"

  TEST_OUTPUT="$(
    HOME="${TEST_HOME}" \
    PATH="${TEST_BIN}:${ORIG_PATH}" \
    EDITOR="${TEST_BIN}/fake-editor" \
    "${ROOT_DIR}/bin/ccs" profile edit api 2>&1
  )"
  TEST_STATUS=$?

  assert_status 0
  assert_file_exists "${TEST_HOME}/.config/ccs/profiles/api.env"
  assert_file_contains "${TEST_HOME}/.config/ccs/profiles/api.env" "CLAUDE_CONFIG_DIR=${TEST_HOME}/.config/ccs/claude/api"
  assert_file_contains "${TEST_HOME}/.config/ccs/profiles/api.env" "# edited by test"
}

test_eval_use_then_run_prefers_session_profile_over_global() {
  write_profile "api" \
    "CLAUDE_CONFIG_DIR=${TEST_HOME}/.config/ccs/claude/api" \
    "ANTHROPIC_API_KEY=session-key"
  write_profile "max" \
    "CLAUDE_CONFIG_DIR=${TEST_HOME}/.config/ccs/claude/max"
  write_config "default_profile=max"

  TEST_OUTPUT="$(
    HOME="${TEST_HOME}" \
    PATH="${TEST_BIN}:${ORIG_PATH}" \
    ROOT_DIR="${ROOT_DIR}" \
    bash -c 'eval "$("$ROOT_DIR/bin/ccs" use api)"; "$ROOT_DIR/bin/ccs" run -- --print session'
  )"
  TEST_STATUS=$?

  assert_status 0
  assert_output_contains "CCS_ACTIVE_PROFILE=api"
  assert_output_contains "ANTHROPIC_API_KEY=session-key"
  assert_output_not_contains "CLAUDE_CONFIG_DIR=${TEST_HOME}/.config/ccs/claude/max"
}

test_use_run_sugar_invokes_claude() {
  write_profile "glm" \
    "CLAUDE_CONFIG_DIR=${TEST_HOME}/.config/ccs/claude/glm" \
    "ANTHROPIC_BASE_URL=https://example.test" \
    "ANTHROPIC_AUTH_TOKEN=glm-token"

  run_ccs use glm --run

  assert_status 0
  assert_output_contains "CCS_ACTIVE_PROFILE=glm"
  assert_output_contains "CLAUDE_CONFIG_DIR=${TEST_HOME}/.config/ccs/claude/glm"
}

test_installed_ccs_init_can_install_hooks() {
  run_install
  assert_status 0

  rm -f "${TEST_HOME}/.config/ccs/ccs.sh" "${TEST_HOME}/.zshrc" "${TEST_HOME}/.bashrc"

  TEST_OUTPUT="$(
    printf 'n\nn\nn\nn\ny\n' | \
      HOME="${TEST_HOME}" \
      PATH="${TEST_BIN}:${TEST_HOME}/.local/bin:${ORIG_PATH}" \
      "${TEST_HOME}/.local/bin/ccs" init 2>&1
  )"
  TEST_STATUS=$?

  assert_status 0
  assert_file_exists "${TEST_HOME}/.config/ccs/ccs.sh"
  assert_file_contains "${TEST_HOME}/.zshrc" "ccs shell hook"
  assert_file_contains "${TEST_HOME}/.bashrc" "ccs shell hook"
  assert_output_not_contains "skipped"
}

test_manual_profile_extra_vars_are_applied() {
  write_profile "glm" \
    "CLAUDE_CONFIG_DIR=${TEST_HOME}/.config/ccs/claude/glm" \
    "CCS_SHARED_CLAUDE_DIR=${TEST_HOME}/.claude" \
    "ANTHROPIC_BASE_URL=https://example.test" \
    "ANTHROPIC_AUTH_TOKEN=glm-token" \
    "ANTHROPIC_DEFAULT_OPUS_MODEL=glm-5.1" \
    "ANTHROPIC_DEFAULT_SONNET_MODEL=glm-4.7" \
    "ANTHROPIC_DEFAULT_HAIKU_MODEL=glm-4.5-air"

  run_ccs run glm -- --print models

  assert_status 0
  assert_output_contains "ANTHROPIC_DEFAULT_SONNET_MODEL=glm-4.7"
  assert_output_contains "ANTHROPIC_DEFAULT_HAIKU_MODEL=glm-4.5-air"
}

test_run_creates_default_shared_symlinks() {
  setup_shared_claude_base
  write_profile "max" \
    "CLAUDE_CONFIG_DIR=${TEST_HOME}/.config/ccs/claude/max" \
    "CCS_SHARED_CLAUDE_DIR=${TEST_HOME}/.claude" \
    "CCS_SHARED_PATHS=CLAUDE.md,settings.json,skills,plugins,rules"

  run_ccs run max -- --print shared

  assert_status 0
  assert_file_exists "${TEST_HOME}/.config/ccs/claude/max"
  assert_equals "${TEST_HOME}/.claude/settings.json" "$(readlink "${TEST_HOME}/.config/ccs/claude/max/settings.json")"
  assert_equals "${TEST_HOME}/.claude/skills" "$(readlink "${TEST_HOME}/.config/ccs/claude/max/skills")"
  assert_equals "${TEST_HOME}/.claude/plugins" "$(readlink "${TEST_HOME}/.config/ccs/claude/max/plugins")"
  assert_equals "${TEST_HOME}/.claude/rules" "$(readlink "${TEST_HOME}/.config/ccs/claude/max/rules")"
  assert_equals "${TEST_HOME}/.claude/CLAUDE.md" "$(readlink "${TEST_HOME}/.config/ccs/claude/max/CLAUDE.md")"
}

test_run_uses_implicit_default_shared_config_for_existing_profiles() {
  setup_shared_claude_base
  write_profile "api" \
    "CLAUDE_CONFIG_DIR=${TEST_HOME}/.config/ccs/claude/api" \
    "ANTHROPIC_API_KEY=test-key"

  run_ccs run api -- --print shared-default

  assert_status 0
  assert_equals "${TEST_HOME}/.claude/settings.json" "$(readlink "${TEST_HOME}/.config/ccs/claude/api/settings.json")"
  assert_equals "${TEST_HOME}/.claude/skills" "$(readlink "${TEST_HOME}/.config/ccs/claude/api/skills")"
  assert_equals "${TEST_HOME}/.claude/plugins" "$(readlink "${TEST_HOME}/.config/ccs/claude/api/plugins")"
  assert_equals "${TEST_HOME}/.claude/rules" "$(readlink "${TEST_HOME}/.config/ccs/claude/api/rules")"
  assert_equals "${TEST_HOME}/.claude/CLAUDE.md" "$(readlink "${TEST_HOME}/.config/ccs/claude/api/CLAUDE.md")"
}

test_existing_local_shared_paths_are_backed_up_and_relinked() {
  setup_shared_claude_base
  mkdir -p "${TEST_HOME}/.config/ccs/claude/glm"
  printf 'local settings\n' > "${TEST_HOME}/.config/ccs/claude/glm/settings.json"
  mkdir -p "${TEST_HOME}/.config/ccs/claude/glm/plugins"
  printf 'local plugin\n' > "${TEST_HOME}/.config/ccs/claude/glm/plugins/local.txt"
  write_profile "glm" \
    "CLAUDE_CONFIG_DIR=${TEST_HOME}/.config/ccs/claude/glm" \
    "CCS_SHARED_CLAUDE_DIR=${TEST_HOME}/.claude" \
    "CCS_SHARED_PATHS=settings.json,plugins" \
    "ANTHROPIC_BASE_URL=https://example.test" \
    "ANTHROPIC_AUTH_TOKEN=glm-token"

  run_ccs run glm -- --print migrate

  assert_status 0
  assert_equals "${TEST_HOME}/.claude/settings.json" "$(readlink "${TEST_HOME}/.config/ccs/claude/glm/settings.json")"
  assert_equals "${TEST_HOME}/.claude/plugins" "$(readlink "${TEST_HOME}/.config/ccs/claude/glm/plugins")"
  assert_file_contains "${TEST_HOME}/.config/ccs/claude/glm/.ccs-local-backup/settings.json" "local settings"
  assert_file_contains "${TEST_HOME}/.config/ccs/claude/glm/.ccs-local-backup/plugins/local.txt" "local plugin"
}

test_run_uses_mimo_profile() {
  write_profile "mimo" \
    "CLAUDE_CONFIG_DIR=${TEST_HOME}/.config/ccs/claude/mimo" \
    "ANTHROPIC_BASE_URL=https://api.xiaomimimo.com/anthropic" \
    "ANTHROPIC_AUTH_TOKEN=sk-mimo-test" \
    "ANTHROPIC_DEFAULT_OPUS_MODEL=mimo-v2.5-pro" \
    "ANTHROPIC_DEFAULT_SONNET_MODEL=mimo-v2.5" \
    "ANTHROPIC_DEFAULT_HAIKU_MODEL=mimo-v2.5"

  run_ccs run mimo -- --print mimo

  assert_status 0
  assert_output_contains "CCS_ACTIVE_PROFILE=mimo"
  assert_output_contains "ANTHROPIC_BASE_URL=https://api.xiaomimimo.com/anthropic"
  assert_output_contains "ANTHROPIC_AUTH_TOKEN=sk-mimo-test"
  assert_output_contains "ANTHROPIC_DEFAULT_OPUS_MODEL=mimo-v2.5-pro"
  assert_output_contains "ANTHROPIC_DEFAULT_SONNET_MODEL=mimo-v2.5"
  assert_output_contains "ANTHROPIC_DEFAULT_HAIKU_MODEL=mimo-v2.5"
}

test_use_mimo_auto_prompts_for_api_key_when_unconfigured() {
  TEST_OUTPUT="$(
    printf 'sk-mimo-auto\n' | \
      HOME="${TEST_HOME}" \
      PATH="${TEST_BIN}:${ORIG_PATH}" \
      "${ROOT_DIR}/bin/ccs" use mimo 2>&1
  )"
  TEST_STATUS=$?

  assert_status 0
  assert_file_exists "${TEST_HOME}/.config/ccs/profiles/mimo.env"
  assert_file_contains "${TEST_HOME}/.config/ccs/profiles/mimo.env" "ANTHROPIC_AUTH_TOKEN=sk-mimo-auto"
  assert_file_contains "${TEST_HOME}/.config/ccs/profiles/mimo.env" "ANTHROPIC_BASE_URL=https://api.xiaomimimo.com/anthropic"
  assert_file_contains "${TEST_HOME}/.config/ccs/profiles/mimo.env" "ANTHROPIC_DEFAULT_OPUS_MODEL=mimo-v2.5-pro"
  assert_output_contains "export ANTHROPIC_AUTH_TOKEN=sk-mimo-auto"
  assert_output_contains "export CCS_ACTIVE_PROFILE=mimo"
}

test_use_mimo_token_plan_defaults_to_sgp_when_blank() {
  TEST_OUTPUT="$(
    printf 'tp-plan-key\n\n' | \
      HOME="${TEST_HOME}" \
      PATH="${TEST_BIN}:${ORIG_PATH}" \
      "${ROOT_DIR}/bin/ccs" use mimo 2>&1
  )"
  TEST_STATUS=$?

  assert_status 0
  assert_file_contains "${TEST_HOME}/.config/ccs/profiles/mimo.env" "ANTHROPIC_BASE_URL=https://token-plan-sgp.xiaomimimo.com/anthropic"
  assert_file_contains "${TEST_HOME}/.config/ccs/profiles/mimo.env" "ANTHROPIC_AUTH_TOKEN=tp-plan-key"
}

test_use_mimo_token_plan_accepts_custom_base_url() {
  TEST_OUTPUT="$(
    printf 'tp-plan-key\nhttps://token-plan-cn.xiaomimimo.com/anthropic\n' | \
      HOME="${TEST_HOME}" \
      PATH="${TEST_BIN}:${ORIG_PATH}" \
      "${ROOT_DIR}/bin/ccs" use mimo 2>&1
  )"
  TEST_STATUS=$?

  assert_status 0
  assert_file_contains "${TEST_HOME}/.config/ccs/profiles/mimo.env" "ANTHROPIC_BASE_URL=https://token-plan-cn.xiaomimimo.com/anthropic"
}

test_use_mimo_does_not_reprompt_when_already_configured() {
  write_profile "mimo" \
    "CLAUDE_CONFIG_DIR=${TEST_HOME}/.config/ccs/claude/mimo" \
    "ANTHROPIC_BASE_URL=https://api.xiaomimimo.com/anthropic" \
    "ANTHROPIC_AUTH_TOKEN=sk-existing"

  TEST_OUTPUT="$(
    HOME="${TEST_HOME}" \
    PATH="${TEST_BIN}:${ORIG_PATH}" \
    "${ROOT_DIR}/bin/ccs" use mimo </dev/null 2>&1
  )"
  TEST_STATUS=$?

  assert_status 0
  assert_output_not_contains "is not configured"
  assert_output_contains "export ANTHROPIC_AUTH_TOKEN=sk-existing"
}

test_mimo_profile_edit_creates_stub_with_xiaomi_base_url() {
  cat >"${TEST_BIN}/fake-editor" <<'EOF'
#!/usr/bin/env bash
exit 0
EOF
  chmod +x "${TEST_BIN}/fake-editor"

  TEST_OUTPUT="$(
    HOME="${TEST_HOME}" \
    PATH="${TEST_BIN}:${ORIG_PATH}" \
    EDITOR="${TEST_BIN}/fake-editor" \
    "${ROOT_DIR}/bin/ccs" profile edit mimo 2>&1
  )"
  TEST_STATUS=$?

  assert_status 0
  assert_file_exists "${TEST_HOME}/.config/ccs/profiles/mimo.env"
  assert_file_contains "${TEST_HOME}/.config/ccs/profiles/mimo.env" "ANTHROPIC_BASE_URL=https://api.xiaomimimo.com/anthropic"
  assert_file_contains "${TEST_HOME}/.config/ccs/profiles/mimo.env" "ANTHROPIC_DEFAULT_OPUS_MODEL=mimo-v2.5-pro"
  assert_file_contains "${TEST_HOME}/.config/ccs/profiles/mimo.env" "ANTHROPIC_DEFAULT_SONNET_MODEL=mimo-v2.5"
}

run_test() {
  local test_name="$1"
  local test_status=0
  CURRENT_TEST_FAILED=0
  setup_test_env
  if ! "${test_name}"; then
    test_status=1
  fi
  if (( test_status != 0 || CURRENT_TEST_FAILED != 0 )); then
    printf 'FAIL %s\n' "${test_name}" >&2
    failures=$((failures + 1))
    cleanup_test_env
    return 0
  fi
  cleanup_test_env
  printf 'PASS %s\n' "${test_name}"
}

test_init_bypass_creates_settings_local_json() {
  local project_dir="${TEST_HOME}/project"
  mkdir -p "${project_dir}"

  TEST_OUTPUT="$(
    HOME="${TEST_HOME}" \
    PATH="${TEST_BIN}:${ORIG_PATH}" \
    bash -c "cd '${project_dir}' && '${ROOT_DIR}/bin/ccs' init bypass" 2>&1
  )"
  TEST_STATUS=$?

  assert_status 0
  assert_file_exists "${project_dir}/.claude/settings.local.json"
  assert_file_contains "${project_dir}/.claude/settings.local.json" '"defaultMode": "bypassPermissions"'
  assert_output_contains "Set permissions.defaultMode=bypassPermissions"
}

test_init_bypass_merges_existing_json() {
  local project_dir="${TEST_HOME}/project"
  mkdir -p "${project_dir}/.claude"
  printf '{"someOtherSetting": true}\n' >"${project_dir}/.claude/settings.local.json"

  TEST_OUTPUT="$(
    HOME="${TEST_HOME}" \
    PATH="${TEST_BIN}:${ORIG_PATH}" \
    bash -c "cd '${project_dir}' && '${ROOT_DIR}/bin/ccs' init bypass" 2>&1
  )"
  TEST_STATUS=$?

  assert_status 0
  assert_file_contains "${project_dir}/.claude/settings.local.json" '"defaultMode": "bypassPermissions"'
  assert_file_contains "${project_dir}/.claude/settings.local.json" '"someOtherSetting": true'
}

test_init_bypass_overwrites_existing_default_mode() {
  local project_dir="${TEST_HOME}/project"
  mkdir -p "${project_dir}/.claude"
  printf '{"permissions": {"defaultMode": "default", "allow": []}}\n' >"${project_dir}/.claude/settings.local.json"

  TEST_OUTPUT="$(
    HOME="${TEST_HOME}" \
    PATH="${TEST_BIN}:${ORIG_PATH}" \
    bash -c "cd '${project_dir}' && '${ROOT_DIR}/bin/ccs' init bypass" 2>&1
  )"
  TEST_STATUS=$?

  assert_status 0
  assert_file_contains "${project_dir}/.claude/settings.local.json" '"defaultMode": "bypassPermissions"'
  assert_file_not_contains "${project_dir}/.claude/settings.local.json" '"defaultMode": "default"'
  assert_file_contains "${project_dir}/.claude/settings.local.json" '"allow": []'
}

test_run_uses_deepseek_profile() {
  write_profile "deepseek" \
    "CLAUDE_CONFIG_DIR=${TEST_HOME}/.config/ccs/claude/deepseek" \
    "ANTHROPIC_BASE_URL=https://api.deepseek.com/anthropic" \
    "ANTHROPIC_AUTH_TOKEN=sk-deepseek-test" \
    "ANTHROPIC_DEFAULT_OPUS_MODEL=deepseek-v4-pro" \
    "ANTHROPIC_DEFAULT_SONNET_MODEL=deepseek-v4-pro" \
    "ANTHROPIC_DEFAULT_HAIKU_MODEL=deepseek-v4-flash" \
    "CLAUDE_CODE_SUBAGENT_MODEL=deepseek-v4-flash" \
    "CLAUDE_CODE_EFFORT_LEVEL=max"

  run_ccs run deepseek -- --print deepseek

  assert_status 0
  assert_output_contains "CCS_ACTIVE_PROFILE=deepseek"
  assert_output_contains "ANTHROPIC_BASE_URL=https://api.deepseek.com/anthropic"
  assert_output_contains "ANTHROPIC_AUTH_TOKEN=sk-deepseek-test"
  assert_output_contains "ANTHROPIC_DEFAULT_OPUS_MODEL=deepseek-v4-pro"
  assert_output_contains "ANTHROPIC_DEFAULT_HAIKU_MODEL=deepseek-v4-flash"
}

test_use_deepseek_auto_prompts_for_api_key_when_unconfigured() {
  TEST_OUTPUT="$(
    printf 'sk-deepseek-auto\n' | \
      HOME="${TEST_HOME}" \
      PATH="${TEST_BIN}:${ORIG_PATH}" \
      "${ROOT_DIR}/bin/ccs" use deepseek 2>&1
  )"
  TEST_STATUS=$?

  assert_status 0
  assert_file_exists "${TEST_HOME}/.config/ccs/profiles/deepseek.env"
  assert_file_contains "${TEST_HOME}/.config/ccs/profiles/deepseek.env" "ANTHROPIC_AUTH_TOKEN=sk-deepseek-auto"
  assert_file_contains "${TEST_HOME}/.config/ccs/profiles/deepseek.env" "ANTHROPIC_BASE_URL=https://api.deepseek.com/anthropic"
  assert_file_contains "${TEST_HOME}/.config/ccs/profiles/deepseek.env" "ANTHROPIC_DEFAULT_OPUS_MODEL=deepseek-v4-pro"
  assert_file_contains "${TEST_HOME}/.config/ccs/profiles/deepseek.env" "ANTHROPIC_DEFAULT_HAIKU_MODEL=deepseek-v4-flash"
  assert_file_contains "${TEST_HOME}/.config/ccs/profiles/deepseek.env" "CLAUDE_CODE_SUBAGENT_MODEL=deepseek-v4-flash"
  assert_file_contains "${TEST_HOME}/.config/ccs/profiles/deepseek.env" "CLAUDE_CODE_EFFORT_LEVEL=max"
  assert_output_contains "export ANTHROPIC_AUTH_TOKEN=sk-deepseek-auto"
  assert_output_contains "export CCS_ACTIVE_PROFILE=deepseek"
}

test_ds_alias_resolves_to_deepseek() {
  write_profile "deepseek" \
    "CLAUDE_CONFIG_DIR=${TEST_HOME}/.config/ccs/claude/deepseek" \
    "ANTHROPIC_BASE_URL=https://api.deepseek.com/anthropic" \
    "ANTHROPIC_AUTH_TOKEN=sk-alias-test" \
    "ANTHROPIC_DEFAULT_OPUS_MODEL=deepseek-v4-pro" \
    "ANTHROPIC_DEFAULT_SONNET_MODEL=deepseek-v4-pro" \
    "ANTHROPIC_DEFAULT_HAIKU_MODEL=deepseek-v4-flash" \
    "CLAUDE_CODE_SUBAGENT_MODEL=deepseek-v4-flash" \
    "CLAUDE_CODE_EFFORT_LEVEL=max"

  run_ccs run ds -- --print alias

  assert_status 0
  assert_output_contains "CCS_ACTIVE_PROFILE=deepseek"
  assert_output_contains "ANTHROPIC_AUTH_TOKEN=sk-alias-test"
}

test_deepseek_profile_edit_creates_stub() {
  cat >"${TEST_BIN}/fake-editor" <<'EOF'
#!/usr/bin/env bash
exit 0
EOF
  chmod +x "${TEST_BIN}/fake-editor"

  TEST_OUTPUT="$(
    HOME="${TEST_HOME}" \
    PATH="${TEST_BIN}:${ORIG_PATH}" \
    EDITOR="${TEST_BIN}/fake-editor" \
    "${ROOT_DIR}/bin/ccs" profile edit deepseek 2>&1
  )"
  TEST_STATUS=$?

  assert_status 0
  assert_file_exists "${TEST_HOME}/.config/ccs/profiles/deepseek.env"
  assert_file_contains "${TEST_HOME}/.config/ccs/profiles/deepseek.env" "ANTHROPIC_BASE_URL=https://api.deepseek.com/anthropic"
  assert_file_contains "${TEST_HOME}/.config/ccs/profiles/deepseek.env" "ANTHROPIC_DEFAULT_OPUS_MODEL=deepseek-v4-pro"
  assert_file_contains "${TEST_HOME}/.config/ccs/profiles/deepseek.env" "ANTHROPIC_DEFAULT_HAIKU_MODEL=deepseek-v4-flash"
  assert_file_contains "${TEST_HOME}/.config/ccs/profiles/deepseek.env" "CLAUDE_CODE_SUBAGENT_MODEL=deepseek-v4-flash"
}

run_test test_help_shows_main_commands
run_test test_use_emits_exports_for_api_profile
run_test test_run_uses_explicit_profile_and_invokes_claude
run_test test_use_global_sets_default_profile
run_test test_run_without_profile_uses_global_default
run_test test_status_reports_profiles_without_secrets
run_test test_install_writes_ccs_binary_and_shell_hooks
run_test test_installed_hook_applies_global_profile_to_new_shell
run_test test_init_creates_only_selected_profiles
run_test test_profile_edit_creates_stub_and_invokes_editor
run_test test_eval_use_then_run_prefers_session_profile_over_global
run_test test_use_run_sugar_invokes_claude
run_test test_installed_ccs_init_can_install_hooks
run_test test_manual_profile_extra_vars_are_applied
run_test test_run_creates_default_shared_symlinks
run_test test_run_uses_implicit_default_shared_config_for_existing_profiles
run_test test_existing_local_shared_paths_are_backed_up_and_relinked
run_test test_run_uses_mimo_profile
run_test test_mimo_profile_edit_creates_stub_with_xiaomi_base_url
run_test test_use_mimo_auto_prompts_for_api_key_when_unconfigured
run_test test_use_mimo_token_plan_defaults_to_sgp_when_blank
run_test test_use_mimo_token_plan_accepts_custom_base_url
run_test test_use_mimo_does_not_reprompt_when_already_configured
run_test test_run_uses_deepseek_profile
run_test test_use_deepseek_auto_prompts_for_api_key_when_unconfigured
run_test test_deepseek_profile_edit_creates_stub
run_test test_ds_alias_resolves_to_deepseek
run_test test_init_bypass_creates_settings_local_json
run_test test_init_bypass_merges_existing_json
run_test test_init_bypass_overwrites_existing_default_mode

if (( failures > 0 )); then
  exit 1
fi
