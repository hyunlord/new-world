#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
APPLY_KEY_FIELDS="false"

if [[ "${1:-}" == "--apply-key-fields" ]]; then
  APPLY_KEY_FIELDS="true"
fi

echo "[migration_verify] root=${ROOT_DIR}"

echo "[migration_verify] 1/4 rust workspace tests"
(
  cd "${ROOT_DIR}/rust"
  cargo test -q
)

echo "[migration_verify] 2/4 data localization extraction"
if [[ "${APPLY_KEY_FIELDS}" == "true" ]]; then
  python3 "${ROOT_DIR}/tools/data_localization_extract.py" --project-root "${ROOT_DIR}" --apply-key-fields
else
  python3 "${ROOT_DIR}/tools/data_localization_extract.py" --project-root "${ROOT_DIR}"
fi

echo "[migration_verify] 3/4 localization compile"
python3 "${ROOT_DIR}/tools/localization_compile.py" --project-root "${ROOT_DIR}"

echo "[migration_verify] 4/4 localization strict audit"
python3 "${ROOT_DIR}/tools/localization_audit.py" --project-root "${ROOT_DIR}" --strict

echo "[migration_verify] completed"
