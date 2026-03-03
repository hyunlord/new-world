#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"

if [[ -z "${GODOT_BIN:-}" ]]; then
  echo "[migration_verify_shadow_ci] GODOT_BIN is required" >&2
  exit 1
fi

shadow_report_json="${MIGRATION_SHADOW_REPORT_JSON:-${ROOT_DIR}/tmp/shadow-longrun-report.json}"
verify_report_json="${MIGRATION_VERIFY_REPORT_JSON:-${ROOT_DIR}/tmp/migration-verify-shadow.json}"
shadow_frames="${MIGRATION_SHADOW_LONGRUN_FRAMES:-10000}"
required_min_frames="${MIGRATION_SHADOW_REQUIRED_MIN_FRAMES:-${shadow_frames}}"

mkdir -p "$(dirname "${shadow_report_json}")"
mkdir -p "$(dirname "${verify_report_json}")"

export MIGRATION_SHADOW_GODOT_BIN="${MIGRATION_SHADOW_GODOT_BIN:-${GODOT_BIN}}"
export MIGRATION_SHADOW_LONGRUN_SEED="${MIGRATION_SHADOW_LONGRUN_SEED:-20260302}"
export MIGRATION_SHADOW_LONGRUN_FRAMES="${shadow_frames}"
export MIGRATION_SHADOW_LONGRUN_DELTA="${MIGRATION_SHADOW_LONGRUN_DELTA:-0.1}"
export MIGRATION_SHADOW_LONGRUN_RUNTIME_MODE="${MIGRATION_SHADOW_LONGRUN_RUNTIME_MODE:-rust_shadow}"
export MIGRATION_SHADOW_LONGRUN_REPORT_PATH="${MIGRATION_SHADOW_LONGRUN_REPORT_PATH:-${shadow_report_json}}"
export MIGRATION_SHADOW_REPORT_JSON="${shadow_report_json}"
export MIGRATION_SHADOW_REQUIRED_MIN_FRAMES="${required_min_frames}"
export MIGRATION_VERIFY_REPORT_JSON="${verify_report_json}"

echo "[migration_verify_shadow_ci] root=${ROOT_DIR}"
echo "[migration_verify_shadow_ci] shadow_report=${MIGRATION_SHADOW_REPORT_JSON}"
echo "[migration_verify_shadow_ci] verify_report=${MIGRATION_VERIFY_REPORT_JSON}"

bash "${ROOT_DIR}/tools/migration_verify.sh" --with-shadow-longrun "$@"
