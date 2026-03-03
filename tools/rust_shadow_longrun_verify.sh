#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
GODOT_BIN="${GODOT_BIN:-}"
SHADOW_SEED="${SHADOW_SEED:-20260302}"
SHADOW_FRAMES="${SHADOW_FRAMES:-10000}"
SHADOW_DELTA="${SHADOW_DELTA:-0.1}"
SHADOW_RUNTIME_MODE="${SHADOW_RUNTIME_MODE:-rust_shadow}"
SHADOW_REPORT_PATH="${SHADOW_REPORT_PATH:-}"

usage() {
  cat <<'USAGE'
Usage:
  GODOT_BIN=/path/to/godot4 tools/rust_shadow_longrun_verify.sh

Optional env:
  SHADOW_SEED           default: 20260302
  SHADOW_FRAMES         default: 10000
  SHADOW_DELTA          default: 0.1
  SHADOW_RUNTIME_MODE   default: rust_shadow
  SHADOW_REPORT_PATH    optional explicit report path (user:// or absolute)
USAGE
}

if [[ $# -gt 0 ]]; then
  echo "[shadow-longrun] unexpected arguments" >&2
  usage
  exit 1
fi

if [[ -z "${GODOT_BIN}" ]]; then
  echo "[shadow-longrun] GODOT_BIN is required" >&2
  usage
  exit 1
fi

if ! [[ "${SHADOW_FRAMES}" =~ ^[0-9]+$ ]] || [[ "${SHADOW_FRAMES}" -le 0 ]]; then
  echo "[shadow-longrun] SHADOW_FRAMES must be a positive integer" >&2
  exit 1
fi

if ! [[ "${SHADOW_SEED}" =~ ^[0-9]+$ ]]; then
  echo "[shadow-longrun] SHADOW_SEED must be an integer" >&2
  exit 1
fi

if ! [[ "${SHADOW_DELTA}" =~ ^[0-9]+(\.[0-9]+)?$ ]]; then
  echo "[shadow-longrun] SHADOW_DELTA must be a number" >&2
  exit 1
fi

tmp_output="$(mktemp)"
trap 'rm -f "${tmp_output}"' EXIT

cmd=(
  "${GODOT_BIN}"
  --headless
  --path "${ROOT_DIR}"
  --script "${ROOT_DIR}/tools/rust_shadow_smoke.gd"
  --
  "--seed=${SHADOW_SEED}"
  "--frames=${SHADOW_FRAMES}"
  "--delta=${SHADOW_DELTA}"
  "--runtime-mode=${SHADOW_RUNTIME_MODE}"
)
if [[ -n "${SHADOW_REPORT_PATH}" ]]; then
  cmd+=("--report-path=${SHADOW_REPORT_PATH}")
fi

echo "[shadow-longrun] running: ${cmd[*]}"
"${cmd[@]}" | tee "${tmp_output}"

resolved_report_path="${SHADOW_REPORT_PATH}"
if [[ -z "${resolved_report_path}" ]]; then
  resolved_report_path="$(grep -E '^SHADOW_REPORT_PATH=' "${tmp_output}" | tail -n 1 | sed -E 's/^SHADOW_REPORT_PATH=//')"
fi

if [[ -z "${resolved_report_path}" ]]; then
  echo "[shadow-longrun] unable to resolve report path from smoke output" >&2
  exit 1
fi

if [[ "${resolved_report_path}" == user://* ]]; then
  echo "[shadow-longrun] report path should be globalized absolute path, got: ${resolved_report_path}" >&2
  exit 1
fi

echo "[shadow-longrun] report=${resolved_report_path}"
python3 "${ROOT_DIR}/tools/rust_shadow_cutover_check.py" \
  --report "${resolved_report_path}" \
  --required-min-frames "${SHADOW_FRAMES}"

echo "[shadow-longrun] cutover gate verification passed"
