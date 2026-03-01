#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
APPLY_KEY_FIELDS="false"
STRIP_INLINE_FIELDS="false"

for arg in "$@"; do
  case "${arg}" in
    --apply-key-fields)
      APPLY_KEY_FIELDS="true"
      ;;
    --strip-inline-fields)
      APPLY_KEY_FIELDS="true"
      STRIP_INLINE_FIELDS="true"
      ;;
    *)
      echo "[migration_verify] unknown option: ${arg}" >&2
      echo "usage: tools/migration_verify.sh [--apply-key-fields] [--strip-inline-fields]" >&2
      exit 1
      ;;
  esac
done

echo "[migration_verify] root=${ROOT_DIR}"

echo "[migration_verify] 1/4 rust workspace tests"
(
  cd "${ROOT_DIR}/rust"
  cargo test -q
)

echo "[migration_verify] 2/4 data localization extraction"
extract_cmd=(python3 "${ROOT_DIR}/tools/data_localization_extract.py" --project-root "${ROOT_DIR}")
if [[ "${APPLY_KEY_FIELDS}" == "true" ]]; then
  extract_cmd+=(--apply-key-fields)
fi
if [[ "${STRIP_INLINE_FIELDS}" == "true" ]]; then
  extract_cmd+=(--strip-inline-fields)
fi
"${extract_cmd[@]}"

echo "[migration_verify] 3/4 localization compile"
python3 "${ROOT_DIR}/tools/localization_compile.py" --project-root "${ROOT_DIR}"

echo "[migration_verify] 4/4 localization strict audit"
python3 "${ROOT_DIR}/tools/localization_audit.py" --project-root "${ROOT_DIR}" --strict

echo "[migration_verify] completed"
