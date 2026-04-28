#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
INSTALL_BIN_DIR="${HOME}/.local/bin"
INSTALL_BIN_PATH="${INSTALL_BIN_DIR}/ccs"

install_binary() {
  mkdir -p "${INSTALL_BIN_DIR}"
  cp "${ROOT_DIR}/bin/ccs" "${INSTALL_BIN_PATH}"
  chmod +x "${INSTALL_BIN_PATH}"
}

main() {
  install_binary
  "${INSTALL_BIN_PATH}" internal install-hooks
  printf 'Installed ccs to %s\n' "${INSTALL_BIN_PATH}"
}

main "$@"
