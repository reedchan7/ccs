#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
INSTALL_BIN_DIR="${HOME}/.local/bin"
INSTALL_BIN_PATH="${INSTALL_BIN_DIR}/ccs"

main() {
  cargo build --release --locked --manifest-path "${ROOT_DIR}/Cargo.toml"
  mkdir -p "${INSTALL_BIN_DIR}"
  cp "${ROOT_DIR}/target/release/ccs" "${INSTALL_BIN_PATH}"
  chmod +x "${INSTALL_BIN_PATH}"
  "${INSTALL_BIN_PATH}" init --hooks-only
  printf 'Installed ccs to %s\n' "${INSTALL_BIN_PATH}"
}

main "$@"
