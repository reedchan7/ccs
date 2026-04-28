#!/usr/bin/env bash

mark_assert_failure() {
  CURRENT_TEST_FAILED=1
}

assert_status() {
  local expected="$1"
  if [[ "${TEST_STATUS}" != "${expected}" ]]; then
    printf 'assert_status failed: expected %s got %s\n' "${expected}" "${TEST_STATUS}" >&2
    mark_assert_failure
  fi
}

assert_output_contains() {
  local expected="$1"
  if [[ "${TEST_OUTPUT}" != *"${expected}"* ]]; then
    printf 'assert_output_contains failed: missing %s\n' "${expected}" >&2
    printf 'output was:\n%s\n' "${TEST_OUTPUT}" >&2
    mark_assert_failure
  fi
}

assert_output_not_contains() {
  local unexpected="$1"
  if [[ "${TEST_OUTPUT}" == *"${unexpected}"* ]]; then
    printf 'assert_output_not_contains failed: found %s\n' "${unexpected}" >&2
    printf 'output was:\n%s\n' "${TEST_OUTPUT}" >&2
    mark_assert_failure
  fi
}

assert_file_contains() {
  local file_path="$1"
  local expected="$2"
  if [[ ! -f "${file_path}" ]]; then
    printf 'assert_file_contains failed: missing file %s\n' "${file_path}" >&2
    mark_assert_failure
    return 0
  fi
  if ! grep -Fq -- "${expected}" "${file_path}"; then
    printf 'assert_file_contains failed: %s missing %s\n' "${file_path}" "${expected}" >&2
    mark_assert_failure
  fi
}

assert_file_not_contains() {
  local file_path="$1"
  local unexpected="$2"
  if [[ ! -f "${file_path}" ]]; then
    return 0
  fi
  if grep -Fq -- "${unexpected}" "${file_path}"; then
    printf 'assert_file_not_contains failed: %s contains %s\n' "${file_path}" "${unexpected}" >&2
    mark_assert_failure
  fi
}

assert_file_exists() {
  local file_path="$1"
  if [[ ! -e "${file_path}" ]]; then
    printf 'assert_file_exists failed: missing %s\n' "${file_path}" >&2
    mark_assert_failure
  fi
}

assert_file_not_exists() {
  local file_path="$1"
  if [[ -e "${file_path}" ]]; then
    printf 'assert_file_not_exists failed: unexpectedly found %s\n' "${file_path}" >&2
    mark_assert_failure
  fi
}

assert_equals() {
  local expected="$1"
  local actual="$2"
  if [[ "${expected}" != "${actual}" ]]; then
    printf 'assert_equals failed: expected %s got %s\n' "${expected}" "${actual}" >&2
    mark_assert_failure
  fi
}

run_ccs() {
  local root_dir="${ROOT_DIR:?ROOT_DIR is required}"
  TEST_OUTPUT="$(
    HOME="${TEST_HOME}" \
    PATH="${TEST_BIN}:${ORIG_PATH}" \
    "${root_dir}/bin/ccs" "$@" 2>&1
  )"
  TEST_STATUS=$?
}
