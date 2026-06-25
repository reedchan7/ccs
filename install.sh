#!/usr/bin/env bash
set -euo pipefail

REPO_OWNER="${CCS_REPO_OWNER:-reedchan7}"
REPO_NAME="${CCS_REPO_NAME:-ccs}"
INSTALL_BIN_DIR="${CCS_INSTALL_DIR:-${HOME}/.local/bin}"
INSTALL_BIN_PATH="${INSTALL_BIN_DIR}/ccs"
TMPDIRS=""

cleanup() {
  local dir
  for dir in ${TMPDIRS}; do
    rm -rf "${dir}"
  done
}

trap cleanup EXIT

say() {
  printf '%s\n' "$*"
}

err() {
  printf 'error: %s\n' "$*" >&2
  exit 1
}

need_cmd() {
  command -v "$1" >/dev/null 2>&1 || err "missing required command: $1"
}

make_tmpdir() {
  local dir
  dir="$(mktemp -d)"
  TMPDIRS="${TMPDIRS} ${dir}"
  printf '%s' "${dir}"
}

target_triple() {
  case "$(uname -s)-$(uname -m)" in
    Linux-x86_64) printf 'x86_64-unknown-linux-gnu' ;;
    Linux-aarch64 | Linux-arm64) printf 'aarch64-unknown-linux-gnu' ;;
    Darwin-x86_64) printf 'x86_64-apple-darwin' ;;
    Darwin-arm64) printf 'aarch64-apple-darwin' ;;
    *) err "unsupported platform: $(uname -s) $(uname -m)" ;;
  esac
}

latest_version() {
  local response

  if [ -n "${CCS_VERSION:-}" ]; then
    case "${CCS_VERSION}" in
      v*) printf '%s' "${CCS_VERSION}" ;;
      *) printf 'v%s' "${CCS_VERSION}" ;;
    esac
    return
  fi

  response="$(
    curl -fsSL "https://api.github.com/repos/${REPO_OWNER}/${REPO_NAME}/releases/latest" \
      2>/dev/null || true
  )"
  printf '%s\n' "${response}" \
    | sed -n 's/.*"tag_name"[[:space:]]*:[[:space:]]*"\([^"]*\)".*/\1/p' \
    | head -n 1
}

installed_version() {
  if [ -x "${INSTALL_BIN_PATH}" ]; then
    "${INSTALL_BIN_PATH}" version 2>/dev/null | awk '{print $2}'
  fi
}

install_release() {
  local version="$1"
  local target="$2"
  local asset="ccs-${version}-${target}.tar.gz"
  local url="https://github.com/${REPO_OWNER}/${REPO_NAME}/releases/download/${version}/${asset}"
  local tmpdir archive extracted

  tmpdir="$(make_tmpdir)"
  archive="${tmpdir}/${asset}"

  say "Downloading ${url}"
  curl -fL --progress-bar "${url}" -o "${archive}"
  tar -xzf "${archive}" -C "${tmpdir}"
  extracted="$(find "${tmpdir}" -type f -name ccs | head -n 1)"
  [ -n "${extracted}" ] || err "release archive did not contain ccs"

  mkdir -p "${INSTALL_BIN_DIR}"
  install -m 0755 "${extracted}" "${INSTALL_BIN_PATH}"
}

install_from_source() {
  local root_dir="$1"
  [ -f "${root_dir}/Cargo.toml" ] || err "source directory does not contain Cargo.toml"
  need_cmd cargo

  cargo build --release --locked --manifest-path "${root_dir}/Cargo.toml"
  mkdir -p "${INSTALL_BIN_DIR}"
  install -m 0755 "${root_dir}/target/release/ccs" "${INSTALL_BIN_PATH}"
}

local_source_dir() {
  local script_path root_dir
  script_path="${BASH_SOURCE[0]:-}"
  [ -n "${script_path}" ] && [ -f "${script_path}" ] || return 0

  root_dir="$(cd "$(dirname "${script_path}")" 2>/dev/null && pwd || true)"
  if [ -n "${root_dir}" ] && [ -f "${root_dir}/Cargo.toml" ]; then
    printf '%s' "${root_dir}"
  fi
}

install_from_git() {
  local tmpdir source_dir
  need_cmd git
  need_cmd cargo

  tmpdir="$(make_tmpdir)"
  source_dir="${tmpdir}/${REPO_NAME}"

  git clone --depth 1 "https://github.com/${REPO_OWNER}/${REPO_NAME}.git" "${source_dir}"
  install_from_source "${source_dir}"
}

install_fallback_source() {
  local source_dir
  source_dir="$(local_source_dir)"
  if [ -n "${source_dir}" ]; then
    install_from_source "${source_dir}"
  else
    install_from_git
  fi
}

path_rc_file() {
  case "$(basename "${SHELL:-}")" in
    zsh) printf '%s/.zshrc' "${HOME}" ;;
    bash)
      if [ "$(uname -s)" = "Darwin" ] && [ -f "${HOME}/.bash_profile" ]; then
        printf '%s/.bash_profile' "${HOME}"
      else
        printf '%s/.bashrc' "${HOME}"
      fi
      ;;
    *) printf '%s/.profile' "${HOME}" ;;
  esac
}

ensure_path() {
  case ":${PATH}:" in
    *":${INSTALL_BIN_DIR}:"*) return ;;
  esac

  local rc_file path_expr line rc_has_path
  rc_file="$(path_rc_file)"
  mkdir -p "$(dirname "${rc_file}")"

  if [ "${INSTALL_BIN_DIR}" = "${HOME}/.local/bin" ]; then
    path_expr='$HOME/.local/bin'
  else
    path_expr="${INSTALL_BIN_DIR}"
  fi
  line="export PATH=\"${path_expr}:\$PATH\""
  rc_has_path=1
  if grep -Fqs "${INSTALL_BIN_DIR}" "${rc_file}" 2>/dev/null; then
    rc_has_path=0
  elif [ "${INSTALL_BIN_DIR}" = "${HOME}/.local/bin" ] \
    && { grep -Fqs '${HOME}/.local/bin' "${rc_file}" 2>/dev/null \
      || grep -Fqs '$HOME/.local/bin' "${rc_file}" 2>/dev/null; }; then
    rc_has_path=0
  fi

  if [ "${rc_has_path}" -ne 0 ]; then
    {
      printf '\n# ccs\n'
      printf '%s\n' "${line}"
    } >> "${rc_file}"
    say "Added ${INSTALL_BIN_DIR} to PATH in ${rc_file}"
  fi

  say "Restart your shell or run: export PATH=\"${INSTALL_BIN_DIR}:\$PATH\""
}

main() {
  need_cmd curl
  need_cmd tar
  need_cmd awk
  need_cmd sed
  need_cmd find
  need_cmd head
  need_cmd install

  local target version current current_tag
  target="$(target_triple)"
  version="$(latest_version)"

  if [ -z "${version}" ]; then
    say "Could not resolve latest release; building from source"
    install_fallback_source
  else
    current="$(installed_version || true)"
    current_tag=""
    if [ -n "${current}" ]; then
      current_tag="v${current#v}"
    fi

    if [ "${current_tag}" = "${version}" ] && [ -x "${INSTALL_BIN_PATH}" ]; then
      say "ccs ${current} is already installed"
    else
      install_release "${version}" "${target}"
    fi
  fi

  "${INSTALL_BIN_PATH}" init --hooks-only
  ensure_path
  say "Installed ccs to ${INSTALL_BIN_PATH}"
}

main "$@"
