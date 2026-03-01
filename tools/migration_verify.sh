#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
APPLY_KEY_FIELDS="false"
STRIP_INLINE_FIELDS="false"
WITH_BENCHES="false"
VERIFY_STARTED_EPOCH="$(date +%s)"
VERIFY_STARTED_AT_UTC="$(date -u +"%Y-%m-%dT%H:%M:%SZ")"
STEP_TESTS_DURATION=0
STEP_EXTRACT_DURATION=0
STEP_COMPILE_DURATION=0
STEP_AUDIT_DURATION=0
STEP_BENCH_DURATION=0

for arg in "$@"; do
  case "${arg}" in
    --apply-key-fields)
      APPLY_KEY_FIELDS="true"
      ;;
    --strip-inline-fields)
      APPLY_KEY_FIELDS="true"
      STRIP_INLINE_FIELDS="true"
      ;;
    --with-benches)
      WITH_BENCHES="true"
      ;;
    *)
      echo "[migration_verify] unknown option: ${arg}" >&2
      echo "usage: tools/migration_verify.sh [--apply-key-fields] [--strip-inline-fields] [--with-benches]" >&2
      exit 1
      ;;
  esac
done

echo "[migration_verify] root=${ROOT_DIR}"

echo "[migration_verify] 1/4 rust workspace tests"
step_started_epoch="$(date +%s)"
(
  cd "${ROOT_DIR}/rust"
  cargo test -q
)
STEP_TESTS_DURATION=$(( $(date +%s) - step_started_epoch ))

echo "[migration_verify] 2/4 data localization extraction"
step_started_epoch="$(date +%s)"
extract_cmd=(python3 "${ROOT_DIR}/tools/data_localization_extract.py" --project-root "${ROOT_DIR}")
if [[ "${APPLY_KEY_FIELDS}" == "true" ]]; then
  extract_cmd+=(--apply-key-fields)
fi
if [[ "${STRIP_INLINE_FIELDS}" == "true" ]]; then
  extract_cmd+=(--strip-inline-fields)
fi
"${extract_cmd[@]}"
STEP_EXTRACT_DURATION=$(( $(date +%s) - step_started_epoch ))

echo "[migration_verify] 3/4 localization compile"
step_started_epoch="$(date +%s)"
compile_report_json="${MIGRATION_COMPILE_REPORT_JSON:-}"
compile_report_dir="${MIGRATION_AUDIT_REPORT_DIR:-}"
if [[ -n "${compile_report_dir}" && -z "${compile_report_json}" ]]; then
  compile_report_dir_prefix="${compile_report_dir%/}"
  if [[ -z "${compile_report_dir_prefix}" ]]; then
    compile_report_dir_prefix="${compile_report_dir}"
  fi
  compile_report_json="${compile_report_dir_prefix}/compile_report.json"
fi
compile_cmd=(
  python3 "${ROOT_DIR}/tools/localization_compile.py"
  --project-root "${ROOT_DIR}"
)
if [[ -n "${compile_report_json}" ]]; then
  compile_cmd+=(--report-json "${compile_report_json}")
fi
"${compile_cmd[@]}"
STEP_COMPILE_DURATION=$(( $(date +%s) - step_started_epoch ))

echo "[migration_verify] 4/4 localization strict audit"
step_started_epoch="$(date +%s)"
audit_report_json="${MIGRATION_AUDIT_REPORT_JSON:-}"
audit_duplicate_report_json="${MIGRATION_AUDIT_DUPLICATE_REPORT_JSON:-}"
audit_conflict_markdown="${MIGRATION_AUDIT_CONFLICT_MARKDOWN:-}"
audit_key_owner_policy_json="${MIGRATION_AUDIT_KEY_OWNER_POLICY:-}"
audit_owner_policy_markdown="${MIGRATION_AUDIT_OWNER_POLICY_MARKDOWN:-}"
audit_owner_policy_compare_report_json="${MIGRATION_AUDIT_OWNER_POLICY_COMPARE_REPORT_JSON:-}"
audit_compare_key_owner_policy="${MIGRATION_AUDIT_COMPARE_KEY_OWNER_POLICY:-}"
audit_refresh_key_owner_policy="${MIGRATION_AUDIT_REFRESH_KEY_OWNER_POLICY:-false}"
audit_report_dir="${MIGRATION_AUDIT_REPORT_DIR:-}"
verify_report_json="${MIGRATION_VERIFY_REPORT_JSON:-}"
verify_assert_artifacts="${MIGRATION_VERIFY_ASSERT_ARTIFACTS:-false}"
verify_audit_conflict_preview_limit="${MIGRATION_VERIFY_AUDIT_CONFLICT_PREVIEW_LIMIT:-10}"
if [[ "${verify_assert_artifacts}" != "true" && "${verify_assert_artifacts}" != "false" ]]; then
  echo "[migration_verify] MIGRATION_VERIFY_ASSERT_ARTIFACTS must be true or false" >&2
  exit 1
fi
if ! [[ "${verify_audit_conflict_preview_limit}" =~ ^[0-9]+$ ]]; then
  echo "[migration_verify] MIGRATION_VERIFY_AUDIT_CONFLICT_PREVIEW_LIMIT must be a non-negative integer" >&2
  exit 1
fi
if [[ -n "${audit_report_dir}" ]]; then
  audit_report_dir_prefix="${audit_report_dir%/}"
  if [[ -z "${audit_report_dir_prefix}" ]]; then
    audit_report_dir_prefix="${audit_report_dir}"
  fi
  if [[ -z "${audit_report_json}" ]]; then
    audit_report_json="${audit_report_dir_prefix}/audit.json"
  fi
  if [[ -z "${audit_duplicate_report_json}" ]]; then
    audit_duplicate_report_json="${audit_report_dir_prefix}/duplicate.json"
  fi
  if [[ -z "${audit_conflict_markdown}" ]]; then
    audit_conflict_markdown="${audit_report_dir_prefix}/duplicate_conflicts.md"
  fi
  if [[ -z "${audit_key_owner_policy_json}" ]]; then
    audit_key_owner_policy_json="${audit_report_dir_prefix}/key_owner_policy.generated.json"
  fi
  if [[ -z "${audit_owner_policy_markdown}" ]]; then
    audit_owner_policy_markdown="${audit_report_dir_prefix}/owner_policy.md"
  fi
  if [[ -z "${audit_owner_policy_compare_report_json}" ]]; then
    audit_owner_policy_compare_report_json="${audit_report_dir_prefix}/owner_policy_compare.json"
  fi
  if [[ -z "${verify_report_json}" ]]; then
    verify_report_json="${audit_report_dir_prefix}/migration_verify_report.json"
  fi
  echo "[migration_verify] audit artifact dir=${audit_report_dir_prefix}"
fi
audit_cmd=(
  python3 "${ROOT_DIR}/tools/localization_audit.py"
  --project-root "${ROOT_DIR}"
  --strict
  --compare-key-owner-policy-auto
)
if [[ "${audit_refresh_key_owner_policy}" != "true" && "${audit_refresh_key_owner_policy}" != "false" ]]; then
  echo "[migration_verify] MIGRATION_AUDIT_REFRESH_KEY_OWNER_POLICY must be true or false" >&2
  exit 1
fi
if [[ -n "${audit_report_json}" ]]; then
  audit_cmd+=(--report-json "${audit_report_json}")
fi
if [[ -n "${audit_duplicate_report_json}" ]]; then
  audit_cmd+=(--duplicate-report-json "${audit_duplicate_report_json}")
fi
if [[ -n "${audit_conflict_markdown}" ]]; then
  audit_cmd+=(--duplicate-conflict-markdown "${audit_conflict_markdown}")
fi
if [[ -n "${audit_key_owner_policy_json}" ]]; then
  audit_cmd+=(--key-owner-policy-json "${audit_key_owner_policy_json}")
fi
if [[ -n "${audit_owner_policy_markdown}" ]]; then
  audit_cmd+=(--owner-policy-markdown "${audit_owner_policy_markdown}")
fi
if [[ -n "${audit_owner_policy_compare_report_json}" ]]; then
  audit_cmd+=(--owner-policy-compare-report-json "${audit_owner_policy_compare_report_json}")
fi
if [[ "${audit_refresh_key_owner_policy}" == "true" ]]; then
  audit_cmd+=(--refresh-key-owner-policy-auto)
fi
if [[ -n "${audit_compare_key_owner_policy}" ]]; then
  # Explicit path overrides auto-manifest compare target.
  audit_cmd=(
    python3 "${ROOT_DIR}/tools/localization_audit.py"
    --project-root "${ROOT_DIR}"
    --strict
  )
  if [[ -n "${audit_report_json}" ]]; then
    audit_cmd+=(--report-json "${audit_report_json}")
  fi
  if [[ -n "${audit_duplicate_report_json}" ]]; then
    audit_cmd+=(--duplicate-report-json "${audit_duplicate_report_json}")
  fi
  if [[ -n "${audit_conflict_markdown}" ]]; then
    audit_cmd+=(--duplicate-conflict-markdown "${audit_conflict_markdown}")
  fi
  if [[ -n "${audit_key_owner_policy_json}" ]]; then
    audit_cmd+=(--key-owner-policy-json "${audit_key_owner_policy_json}")
  fi
  if [[ -n "${audit_owner_policy_markdown}" ]]; then
    audit_cmd+=(--owner-policy-markdown "${audit_owner_policy_markdown}")
  fi
  if [[ -n "${audit_owner_policy_compare_report_json}" ]]; then
    audit_cmd+=(--owner-policy-compare-report-json "${audit_owner_policy_compare_report_json}")
  fi
  if [[ "${audit_refresh_key_owner_policy}" == "true" ]]; then
    audit_cmd+=(--refresh-key-owner-policy-auto)
  fi
  audit_cmd+=(--compare-key-owner-policy "${audit_compare_key_owner_policy}")
fi
"${audit_cmd[@]}"
STEP_AUDIT_DURATION=$(( $(date +%s) - step_started_epoch ))

bench_report_json="${MIGRATION_BENCH_REPORT_JSON:-}"
if [[ "${WITH_BENCHES}" == "true" ]]; then
  step_started_epoch="$(date +%s)"
  echo "[migration_verify] 5/5 rust bench checksum verification"
  path_iters="${MIGRATION_BENCH_PATH_ITERS:-100}"
  stress_iters="${MIGRATION_BENCH_STRESS_ITERS:-10000}"
  needs_iters="${MIGRATION_BENCH_NEEDS_ITERS:-10000}"
  path_split="${MIGRATION_BENCH_PATH_SPLIT:-false}"
  path_backend="${MIGRATION_BENCH_PATH_BACKEND:-auto}"
  path_backend_smoke="${MIGRATION_BENCH_PATH_BACKEND_SMOKE:-false}"
  path_backend_smoke_iters="${MIGRATION_BENCH_PATH_BACKEND_SMOKE_ITERS:-10}"
  path_backend_smoke_expect_has_gpu="${MIGRATION_BENCH_PATH_BACKEND_SMOKE_EXPECT_HAS_GPU:-}"
  path_backend_smoke_expect_auto="${MIGRATION_BENCH_PATH_BACKEND_SMOKE_EXPECT_AUTO_RESOLVED:-}"
  path_backend_smoke_expect_gpu="${MIGRATION_BENCH_PATH_BACKEND_SMOKE_EXPECT_GPU_RESOLVED:-}"
  expected_resolved_backend="${MIGRATION_BENCH_EXPECT_RESOLVED_BACKEND:-}"
  bench_report_json="${MIGRATION_BENCH_REPORT_JSON:-}"
  if [[ -n "${audit_report_dir}" && -z "${bench_report_json}" ]]; then
    bench_report_json="${audit_report_dir_prefix}/bench_report.json"
  fi
  LAST_BENCH_CHECKSUM=""
  PATH_SPLIT_TUPLE_CHECKSUM=""
  PATH_SPLIT_XY_CHECKSUM=""
  PATH_SPLIT_TUPLE_TOTAL=""
  PATH_SPLIT_XY_TOTAL=""
  PATH_SMOKE_CHECKSUM=""
  PATH_SMOKE_TOTAL=""
  PATH_SMOKE_RESOLVED_AUTO=""
  PATH_SMOKE_RESOLVED_GPU=""
  PATH_SMOKE_HAS_GPU=""
  STRESS_CHECKSUM=""
  NEEDS_CHECKSUM=""
  for value in "${path_iters}" "${stress_iters}" "${needs_iters}" "${path_backend_smoke_iters}"; do
    if ! [[ "${value}" =~ ^[0-9]+$ ]] || [[ "${value}" -le 0 ]]; then
      echo "[migration_verify] bench iterations must be positive integers" >&2
      exit 1
    fi
  done
  if [[ "${path_split}" != "true" && "${path_split}" != "false" ]]; then
    echo "[migration_verify] MIGRATION_BENCH_PATH_SPLIT must be true or false" >&2
    exit 1
  fi
  if [[ "${path_backend}" != "auto" && "${path_backend}" != "cpu" && "${path_backend}" != "gpu" ]]; then
    echo "[migration_verify] MIGRATION_BENCH_PATH_BACKEND must be auto, cpu, or gpu" >&2
    exit 1
  fi
  if [[ "${path_backend_smoke}" != "true" && "${path_backend_smoke}" != "false" ]]; then
    echo "[migration_verify] MIGRATION_BENCH_PATH_BACKEND_SMOKE must be true or false" >&2
    exit 1
  fi
  if [[ -n "${expected_resolved_backend}" && "${expected_resolved_backend}" != "cpu" && "${expected_resolved_backend}" != "gpu" ]]; then
    echo "[migration_verify] MIGRATION_BENCH_EXPECT_RESOLVED_BACKEND must be cpu or gpu" >&2
    exit 1
  fi
  if [[ -n "${path_backend_smoke_expect_auto}" && "${path_backend_smoke_expect_auto}" != "cpu" && "${path_backend_smoke_expect_auto}" != "gpu" ]]; then
    echo "[migration_verify] MIGRATION_BENCH_PATH_BACKEND_SMOKE_EXPECT_AUTO_RESOLVED must be cpu or gpu" >&2
    exit 1
  fi
  if [[ -n "${path_backend_smoke_expect_has_gpu}" && "${path_backend_smoke_expect_has_gpu}" != "true" && "${path_backend_smoke_expect_has_gpu}" != "false" ]]; then
    echo "[migration_verify] MIGRATION_BENCH_PATH_BACKEND_SMOKE_EXPECT_HAS_GPU must be true or false" >&2
    exit 1
  fi
  if [[ -n "${path_backend_smoke_expect_gpu}" && "${path_backend_smoke_expect_gpu}" != "cpu" && "${path_backend_smoke_expect_gpu}" != "gpu" ]]; then
    echo "[migration_verify] MIGRATION_BENCH_PATH_BACKEND_SMOKE_EXPECT_GPU_RESOLVED must be cpu or gpu" >&2
    exit 1
  fi
  echo "[migration_verify] bench iters: path=${path_iters} stress=${stress_iters} needs=${needs_iters} split=${path_split} path_backend=${path_backend} smoke=${path_backend_smoke} smoke_iters=${path_backend_smoke_iters} smoke_expect_has_gpu=${path_backend_smoke_expect_has_gpu:-none} smoke_expect_auto=${path_backend_smoke_expect_auto:-none} smoke_expect_gpu=${path_backend_smoke_expect_gpu:-none} expected_resolved=${expected_resolved_backend:-none}"

  run_bench_and_check() {
    local name="$1"
    local expected_checksum="$2"
    shift 2

    local output
    output="$("$@")"
    echo "${output}"

    local checksum
    checksum="$(echo "${output}" | sed -n 's/.*checksum=\([0-9.]*\).*/\1/p' | tail -n 1)"
    if [[ -z "${checksum}" ]]; then
      echo "[migration_verify] ${name} checksum parse failed" >&2
      exit 1
    fi
    if [[ "${checksum}" != "${expected_checksum}" ]]; then
      echo "[migration_verify] ${name} checksum mismatch: expected=${expected_checksum} got=${checksum}" >&2
      exit 1
    fi
    LAST_BENCH_CHECKSUM="${checksum}"
    echo "[migration_verify] ${name} checksum ok: ${checksum}"
  }

  run_bench_observe() {
    local name="$1"
    shift
    local output
    output="$("$@")"
    echo "${output}"

    local checksum
    checksum="$(echo "${output}" | sed -n 's/.*checksum=\([0-9.]*\).*/\1/p' | tail -n 1)"
    if [[ -z "${checksum}" ]]; then
      echo "[migration_verify] ${name} checksum parse failed" >&2
      exit 1
    fi
    LAST_BENCH_CHECKSUM="${checksum}"
    echo "[migration_verify] ${name} checksum observed (non-default iters): ${checksum}"
  }

  run_path_split_observe() {
    local output
    output="$("$@")"
    echo "${output}"

    local checksum_lines
    checksum_lines="$(echo "${output}" | sed -n 's/.*checksum=\([0-9.]*\).*/\1/p')"
    local tuple_checksum
    local xy_checksum
    local total_lines
    local tuple_total
    local xy_total
    tuple_checksum="$(echo "${checksum_lines}" | sed -n '1p')"
    xy_checksum="$(echo "${checksum_lines}" | sed -n '2p')"
    total_lines="$(echo "${output}" | sed -n 's/.*total=\([0-9][0-9]*\).*/\1/p')"
    tuple_total="$(echo "${total_lines}" | sed -n '1p')"
    xy_total="$(echo "${total_lines}" | sed -n '2p')"
    if [[ -z "${tuple_checksum}" || -z "${xy_checksum}" ]]; then
      echo "[migration_verify] pathfind-bridge-split checksum parse failed" >&2
      exit 1
    fi
    if [[ -z "${tuple_total}" || -z "${xy_total}" ]]; then
      echo "[migration_verify] pathfind-bridge-split dispatch total parse failed" >&2
      exit 1
    fi
    if [[ "${tuple_checksum}" != "${xy_checksum}" ]]; then
      echo "[migration_verify] pathfind-bridge-split checksum mismatch: tuple=${tuple_checksum} xy=${xy_checksum}" >&2
      exit 1
    fi
    if [[ "${tuple_total}" != "${path_iters}" || "${xy_total}" != "${path_iters}" ]]; then
      echo "[migration_verify] pathfind-bridge-split dispatch total mismatch: expected_each=${path_iters} got_tuple=${tuple_total} got_xy=${xy_total}" >&2
      exit 1
    fi
    if [[ -n "${expected_resolved_backend}" ]]; then
      local resolved_lines
      resolved_lines="$(echo "${output}" | sed -n 's/.*resolved=\([a-z]*\).*/\1/p')"
      if [[ -z "${resolved_lines}" ]]; then
        echo "[migration_verify] pathfind-bridge-split resolved backend parse failed" >&2
        exit 1
      fi
      local resolved
      while IFS= read -r resolved; do
        if [[ "${resolved}" != "${expected_resolved_backend}" ]]; then
          echo "[migration_verify] pathfind-bridge-split resolved backend mismatch: expected=${expected_resolved_backend} got=${resolved}" >&2
          exit 1
        fi
      done <<< "${resolved_lines}"
      echo "[migration_verify] pathfind-bridge-split resolved backend ok: ${expected_resolved_backend}"
    fi
    PATH_SPLIT_TUPLE_CHECKSUM="${tuple_checksum}"
    PATH_SPLIT_XY_CHECKSUM="${xy_checksum}"
    PATH_SPLIT_TUPLE_TOTAL="${tuple_total}"
    PATH_SPLIT_XY_TOTAL="${xy_total}"
    echo "[migration_verify] pathfind-bridge-split dispatch totals ok: tuple=${tuple_total} xy=${xy_total}"
    echo "[migration_verify] pathfind-bridge-split checksums observed: tuple=${tuple_checksum} xy=${xy_checksum}"
  }

  run_path_split_and_check() {
    local expected_tuple="$1"
    local expected_xy="$2"
    shift 2
    local output
    output="$("$@")"
    echo "${output}"

    local checksum_lines
    checksum_lines="$(echo "${output}" | sed -n 's/.*checksum=\([0-9.]*\).*/\1/p')"
    local tuple_checksum
    local xy_checksum
    local total_lines
    local tuple_total
    local xy_total
    tuple_checksum="$(echo "${checksum_lines}" | sed -n '1p')"
    xy_checksum="$(echo "${checksum_lines}" | sed -n '2p')"
    total_lines="$(echo "${output}" | sed -n 's/.*total=\([0-9][0-9]*\).*/\1/p')"
    tuple_total="$(echo "${total_lines}" | sed -n '1p')"
    xy_total="$(echo "${total_lines}" | sed -n '2p')"
    if [[ -z "${tuple_checksum}" || -z "${xy_checksum}" ]]; then
      echo "[migration_verify] pathfind-bridge-split checksum parse failed" >&2
      exit 1
    fi
    if [[ -z "${tuple_total}" || -z "${xy_total}" ]]; then
      echo "[migration_verify] pathfind-bridge-split dispatch total parse failed" >&2
      exit 1
    fi
    if [[ "${tuple_checksum}" != "${expected_tuple}" || "${xy_checksum}" != "${expected_xy}" ]]; then
      echo "[migration_verify] pathfind-bridge-split checksum baseline mismatch: expected_tuple=${expected_tuple} got_tuple=${tuple_checksum} expected_xy=${expected_xy} got_xy=${xy_checksum}" >&2
      exit 1
    fi
    if [[ "${tuple_checksum}" != "${xy_checksum}" ]]; then
      echo "[migration_verify] pathfind-bridge-split checksum mismatch: tuple=${tuple_checksum} xy=${xy_checksum}" >&2
      exit 1
    fi
    if [[ "${tuple_total}" != "${path_iters}" || "${xy_total}" != "${path_iters}" ]]; then
      echo "[migration_verify] pathfind-bridge-split dispatch total mismatch: expected_each=${path_iters} got_tuple=${tuple_total} got_xy=${xy_total}" >&2
      exit 1
    fi
    if [[ -n "${expected_resolved_backend}" ]]; then
      local resolved_lines
      resolved_lines="$(echo "${output}" | sed -n 's/.*resolved=\([a-z]*\).*/\1/p')"
      if [[ -z "${resolved_lines}" ]]; then
        echo "[migration_verify] pathfind-bridge-split resolved backend parse failed" >&2
        exit 1
      fi
      local resolved
      while IFS= read -r resolved; do
        if [[ "${resolved}" != "${expected_resolved_backend}" ]]; then
          echo "[migration_verify] pathfind-bridge-split resolved backend mismatch: expected=${expected_resolved_backend} got=${resolved}" >&2
          exit 1
        fi
      done <<< "${resolved_lines}"
      echo "[migration_verify] pathfind-bridge-split resolved backend ok: ${expected_resolved_backend}"
    fi
    PATH_SPLIT_TUPLE_CHECKSUM="${tuple_checksum}"
    PATH_SPLIT_XY_CHECKSUM="${xy_checksum}"
    PATH_SPLIT_TUPLE_TOTAL="${tuple_total}"
    PATH_SPLIT_XY_TOTAL="${xy_total}"
    echo "[migration_verify] pathfind-bridge-split dispatch totals ok: tuple=${tuple_total} xy=${xy_total}"
    echo "[migration_verify] pathfind-bridge-split checksums ok: tuple=${tuple_checksum} xy=${xy_checksum}"
  }

  run_path_backend_smoke_and_check() {
    local smoke_iters="$1"
    local expect_has_gpu="$2"
    local expect_auto="$3"
    local expect_gpu="$4"
    shift
    shift
    shift
    shift
    local output
    output="$("$@")"
    echo "${output}"

    local line_auto
    local line_cpu
    local line_gpu
    line_auto="$(echo "${output}" | sed -n '/^\[sim-test\] pathfind-backend-smoke: mode=auto /p' | head -n 1)"
    line_cpu="$(echo "${output}" | sed -n '/^\[sim-test\] pathfind-backend-smoke: mode=cpu /p' | head -n 1)"
    line_gpu="$(echo "${output}" | sed -n '/^\[sim-test\] pathfind-backend-smoke: mode=gpu /p' | head -n 1)"
    if [[ -z "${line_auto}" || -z "${line_cpu}" || -z "${line_gpu}" ]]; then
      echo "[migration_verify] pathfind-backend-smoke mode lines missing expected modes(auto/cpu/gpu)" >&2
      exit 1
    fi
    local checksum_auto
    local checksum_cpu
    local checksum_gpu
    checksum_auto="$(echo "${line_auto}" | sed -n 's/.*checksum=\([0-9.]*\).*/\1/p')"
    checksum_cpu="$(echo "${line_cpu}" | sed -n 's/.*checksum=\([0-9.]*\).*/\1/p')"
    checksum_gpu="$(echo "${line_gpu}" | sed -n 's/.*checksum=\([0-9.]*\).*/\1/p')"
    if [[ -z "${checksum_auto}" || -z "${checksum_cpu}" || -z "${checksum_gpu}" ]]; then
      echo "[migration_verify] pathfind-backend-smoke checksum parse failed" >&2
      exit 1
    fi
    if [[ "${checksum_auto}" != "${checksum_cpu}" || "${checksum_cpu}" != "${checksum_gpu}" ]]; then
      echo "[migration_verify] pathfind-backend-smoke checksum mismatch: auto=${checksum_auto} cpu=${checksum_cpu} gpu=${checksum_gpu}" >&2
      exit 1
    fi
    local expected_total
    expected_total=$((smoke_iters * 2))
    local total_auto
    local total_cpu
    local total_gpu
    total_auto="$(echo "${line_auto}" | sed -n 's/.*total=\([0-9][0-9]*\).*/\1/p')"
    total_cpu="$(echo "${line_cpu}" | sed -n 's/.*total=\([0-9][0-9]*\).*/\1/p')"
    total_gpu="$(echo "${line_gpu}" | sed -n 's/.*total=\([0-9][0-9]*\).*/\1/p')"
    if [[ -z "${total_auto}" || -z "${total_cpu}" || -z "${total_gpu}" ]]; then
      echo "[migration_verify] pathfind-backend-smoke dispatch total parse failed" >&2
      exit 1
    fi
    if [[ "${total_auto}" != "${expected_total}" || "${total_cpu}" != "${expected_total}" || "${total_gpu}" != "${expected_total}" ]]; then
      echo "[migration_verify] pathfind-backend-smoke dispatch total mismatch: expected=${expected_total} got_auto=${total_auto} got_cpu=${total_cpu} got_gpu=${total_gpu}" >&2
      exit 1
    fi
    local configured_auto
    local configured_cpu
    local configured_gpu
    local has_gpu_auto
    local has_gpu_cpu
    local has_gpu_gpu
    has_gpu_auto="$(echo "${line_auto}" | sed -n 's/.*has_gpu=\([a-z]*\).*/\1/p')"
    has_gpu_cpu="$(echo "${line_cpu}" | sed -n 's/.*has_gpu=\([a-z]*\).*/\1/p')"
    has_gpu_gpu="$(echo "${line_gpu}" | sed -n 's/.*has_gpu=\([a-z]*\).*/\1/p')"
    if [[ -z "${has_gpu_auto}" || -z "${has_gpu_cpu}" || -z "${has_gpu_gpu}" ]]; then
      echo "[migration_verify] pathfind-backend-smoke has_gpu parse failed" >&2
      exit 1
    fi
    if [[ "${has_gpu_auto}" != "true" && "${has_gpu_auto}" != "false" ]]; then
      echo "[migration_verify] pathfind-backend-smoke has_gpu value invalid: auto=${has_gpu_auto}" >&2
      exit 1
    fi
    if [[ "${has_gpu_cpu}" != "true" && "${has_gpu_cpu}" != "false" ]]; then
      echo "[migration_verify] pathfind-backend-smoke has_gpu value invalid: cpu=${has_gpu_cpu}" >&2
      exit 1
    fi
    if [[ "${has_gpu_gpu}" != "true" && "${has_gpu_gpu}" != "false" ]]; then
      echo "[migration_verify] pathfind-backend-smoke has_gpu value invalid: gpu=${has_gpu_gpu}" >&2
      exit 1
    fi
    if [[ "${has_gpu_auto}" != "${has_gpu_cpu}" || "${has_gpu_cpu}" != "${has_gpu_gpu}" ]]; then
      echo "[migration_verify] pathfind-backend-smoke has_gpu mismatch across modes: auto=${has_gpu_auto} cpu=${has_gpu_cpu} gpu=${has_gpu_gpu}" >&2
      exit 1
    fi
    if [[ -n "${expect_has_gpu}" && "${has_gpu_auto}" != "${expect_has_gpu}" ]]; then
      echo "[migration_verify] pathfind-backend-smoke has_gpu mismatch: expected=${expect_has_gpu} got=${has_gpu_auto}" >&2
      exit 1
    fi
    configured_auto="$(echo "${line_auto}" | sed -n 's/.*configured=\([a-z]*\).*/\1/p')"
    configured_cpu="$(echo "${line_cpu}" | sed -n 's/.*configured=\([a-z]*\).*/\1/p')"
    configured_gpu="$(echo "${line_gpu}" | sed -n 's/.*configured=\([a-z]*\).*/\1/p')"
    if [[ "${configured_auto}" != "auto" || "${configured_cpu}" != "cpu" || "${configured_gpu}" != "gpu" ]]; then
      echo "[migration_verify] pathfind-backend-smoke configured mode mismatch: auto=${configured_auto:-<empty>} cpu=${configured_cpu:-<empty>} gpu=${configured_gpu:-<empty>}" >&2
      exit 1
    fi
    local cpu_auto
    local cpu_cpu
    local cpu_gpu
    local gpu_auto
    local gpu_cpu
    local gpu_gpu
    cpu_auto="$(echo "${line_auto}" | sed -n 's/.* cpu=\([0-9][0-9]*\) gpu=.*/\1/p')"
    cpu_cpu="$(echo "${line_cpu}" | sed -n 's/.* cpu=\([0-9][0-9]*\) gpu=.*/\1/p')"
    cpu_gpu="$(echo "${line_gpu}" | sed -n 's/.* cpu=\([0-9][0-9]*\) gpu=.*/\1/p')"
    gpu_auto="$(echo "${line_auto}" | sed -n 's/.* gpu=\([0-9][0-9]*\) total=.*/\1/p')"
    gpu_cpu="$(echo "${line_cpu}" | sed -n 's/.* gpu=\([0-9][0-9]*\) total=.*/\1/p')"
    gpu_gpu="$(echo "${line_gpu}" | sed -n 's/.* gpu=\([0-9][0-9]*\) total=.*/\1/p')"
    if [[ -z "${cpu_auto}" || -z "${cpu_cpu}" || -z "${cpu_gpu}" || -z "${gpu_auto}" || -z "${gpu_cpu}" || -z "${gpu_gpu}" ]]; then
      echo "[migration_verify] pathfind-backend-smoke cpu/gpu dispatch count parse failed" >&2
      exit 1
    fi
    if [[ $((cpu_auto + gpu_auto)) -ne "${total_auto}" || $((cpu_cpu + gpu_cpu)) -ne "${total_cpu}" || $((cpu_gpu + gpu_gpu)) -ne "${total_gpu}" ]]; then
      echo "[migration_verify] pathfind-backend-smoke dispatch sum mismatch: auto=${cpu_auto}+${gpu_auto}!=$total_auto cpu=${cpu_cpu}+${gpu_cpu}!=$total_cpu gpu=${cpu_gpu}+${gpu_gpu}!=$total_gpu" >&2
      exit 1
    fi
    local resolved_cpu
    local resolved_auto
    local resolved_gpu
    resolved_auto="$(echo "${line_auto}" | sed -n 's/.*resolved=\([a-z]*\).*/\1/p')"
    resolved_cpu="$(echo "${line_cpu}" | sed -n 's/.*resolved=\([a-z]*\).*/\1/p')"
    resolved_gpu="$(echo "${line_gpu}" | sed -n 's/.*resolved=\([a-z]*\).*/\1/p')"
    if [[ "${resolved_cpu}" != "cpu" ]]; then
      echo "[migration_verify] pathfind-backend-smoke cpu mode must resolve to cpu, got=${resolved_cpu:-<empty>}" >&2
      exit 1
    fi
    if [[ -n "${expect_auto}" && "${resolved_auto}" != "${expect_auto}" ]]; then
      echo "[migration_verify] pathfind-backend-smoke auto mode resolve mismatch: expected=${expect_auto} got=${resolved_auto:-<empty>}" >&2
      exit 1
    fi
    if [[ -n "${expect_gpu}" && "${resolved_gpu}" != "${expect_gpu}" ]]; then
      echo "[migration_verify] pathfind-backend-smoke gpu mode resolve mismatch: expected=${expect_gpu} got=${resolved_gpu:-<empty>}" >&2
      exit 1
    fi
    if [[ "${resolved_auto}" == "cpu" && "${gpu_auto}" != "0" ]]; then
      echo "[migration_verify] pathfind-backend-smoke resolved=cpu must have gpu=0 for auto mode, got=${gpu_auto}" >&2
      exit 1
    fi
    if [[ "${resolved_auto}" == "gpu" && "${cpu_auto}" != "0" ]]; then
      echo "[migration_verify] pathfind-backend-smoke resolved=gpu must have cpu=0 for auto mode, got=${cpu_auto}" >&2
      exit 1
    fi
    if [[ "${resolved_cpu}" == "cpu" && "${gpu_cpu}" != "0" ]]; then
      echo "[migration_verify] pathfind-backend-smoke resolved=cpu must have gpu=0 for cpu mode, got=${gpu_cpu}" >&2
      exit 1
    fi
    if [[ "${resolved_cpu}" == "gpu" && "${cpu_cpu}" != "0" ]]; then
      echo "[migration_verify] pathfind-backend-smoke resolved=gpu must have cpu=0 for cpu mode, got=${cpu_cpu}" >&2
      exit 1
    fi
    if [[ "${resolved_gpu}" == "cpu" && "${gpu_gpu}" != "0" ]]; then
      echo "[migration_verify] pathfind-backend-smoke resolved=cpu must have gpu=0 for gpu mode, got=${gpu_gpu}" >&2
      exit 1
    fi
    if [[ "${resolved_gpu}" == "gpu" && "${cpu_gpu}" != "0" ]]; then
      echo "[migration_verify] pathfind-backend-smoke resolved=gpu must have cpu=0 for gpu mode, got=${cpu_gpu}" >&2
      exit 1
    fi
    if [[ "${has_gpu_auto}" == "true" ]]; then
      if [[ "${resolved_auto}" != "gpu" || "${resolved_gpu}" != "gpu" ]]; then
        echo "[migration_verify] pathfind-backend-smoke has_gpu=true expects resolved auto/gpu to be gpu, got_auto=${resolved_auto} got_gpu=${resolved_gpu}" >&2
        exit 1
      fi
    else
      if [[ "${resolved_auto}" != "cpu" || "${resolved_gpu}" != "cpu" ]]; then
        echo "[migration_verify] pathfind-backend-smoke has_gpu=false expects resolved auto/gpu to be cpu, got_auto=${resolved_auto} got_gpu=${resolved_gpu}" >&2
        exit 1
      fi
    fi
    PATH_SMOKE_CHECKSUM="${checksum_auto}"
    PATH_SMOKE_TOTAL="${expected_total}"
    PATH_SMOKE_RESOLVED_AUTO="${resolved_auto}"
    PATH_SMOKE_RESOLVED_GPU="${resolved_gpu}"
    PATH_SMOKE_HAS_GPU="${has_gpu_auto}"
    echo "[migration_verify] pathfind-backend-smoke checksums/dispatch/resolved ok: checksum=${checksum_auto} total_each=${expected_total} has_gpu=${has_gpu_auto} resolved_auto=${resolved_auto:-unknown} resolved_gpu=${resolved_gpu:-unknown}"
  }

  (
    cd "${ROOT_DIR}/rust"
    if [[ "${path_iters}" == "100" ]]; then
      path_output="$(cargo run -q -p sim-test --release -- --bench-pathfind-bridge --iters "${path_iters}" --backend "${path_backend}")"
      echo "${path_output}"
      path_checksum="$(echo "${path_output}" | sed -n 's/.*checksum=\([0-9.]*\).*/\1/p' | head -n 1)"
      path_total="$(echo "${path_output}" | sed -n 's/.*total=\([0-9][0-9]*\).*/\1/p' | head -n 1)"
      path_resolved="$(echo "${path_output}" | sed -n 's/.*resolved=\([a-z]*\).*/\1/p' | head -n 1)"
      if [[ -z "${path_checksum}" ]]; then
        echo "[migration_verify] pathfind-bridge checksum parse failed" >&2
        exit 1
      fi
      if [[ -z "${path_total}" ]]; then
        echo "[migration_verify] pathfind-bridge dispatch total parse failed" >&2
        exit 1
      fi
      if [[ "${path_checksum}" != "70800.00000" ]]; then
        echo "[migration_verify] pathfind-bridge checksum mismatch: expected=70800.00000 got=${path_checksum}" >&2
        exit 1
      fi
      expected_path_total=$((path_iters * 2))
      if [[ "${path_total}" != "${expected_path_total}" ]]; then
        echo "[migration_verify] pathfind-bridge dispatch total mismatch: expected=${expected_path_total} got=${path_total}" >&2
        exit 1
      fi
      if [[ -n "${expected_resolved_backend}" ]]; then
        path_resolved="$(echo "${path_output}" | sed -n 's/.*resolved=\([a-z]*\).*/\1/p' | head -n 1)"
        if [[ -z "${path_resolved}" ]]; then
          echo "[migration_verify] pathfind-bridge resolved backend parse failed" >&2
          exit 1
        fi
        if [[ "${path_resolved}" != "${expected_resolved_backend}" ]]; then
          echo "[migration_verify] pathfind-bridge resolved backend mismatch: expected=${expected_resolved_backend} got=${path_resolved}" >&2
          exit 1
        fi
        echo "[migration_verify] pathfind-bridge resolved backend ok: ${path_resolved}"
      fi
      echo "[migration_verify] pathfind-bridge dispatch total ok: ${path_total}"
      echo "[migration_verify] pathfind-bridge checksum ok: ${path_checksum}"
    else
      path_output="$(cargo run -q -p sim-test --release -- --bench-pathfind-bridge --iters "${path_iters}" --backend "${path_backend}")"
      echo "${path_output}"
      path_checksum="$(echo "${path_output}" | sed -n 's/.*checksum=\([0-9.]*\).*/\1/p' | head -n 1)"
      path_total="$(echo "${path_output}" | sed -n 's/.*total=\([0-9][0-9]*\).*/\1/p' | head -n 1)"
      path_resolved="$(echo "${path_output}" | sed -n 's/.*resolved=\([a-z]*\).*/\1/p' | head -n 1)"
      if [[ -z "${path_checksum}" ]]; then
        echo "[migration_verify] pathfind-bridge checksum parse failed" >&2
        exit 1
      fi
      if [[ -z "${path_total}" ]]; then
        echo "[migration_verify] pathfind-bridge dispatch total parse failed" >&2
        exit 1
      fi
      expected_path_total=$((path_iters * 2))
      if [[ "${path_total}" != "${expected_path_total}" ]]; then
        echo "[migration_verify] pathfind-bridge dispatch total mismatch: expected=${expected_path_total} got=${path_total}" >&2
        exit 1
      fi
      if [[ -n "${expected_resolved_backend}" ]]; then
        path_resolved="$(echo "${path_output}" | sed -n 's/.*resolved=\([a-z]*\).*/\1/p' | head -n 1)"
        if [[ -z "${path_resolved}" ]]; then
          echo "[migration_verify] pathfind-bridge resolved backend parse failed" >&2
          exit 1
        fi
        if [[ "${path_resolved}" != "${expected_resolved_backend}" ]]; then
          echo "[migration_verify] pathfind-bridge resolved backend mismatch: expected=${expected_resolved_backend} got=${path_resolved}" >&2
          exit 1
        fi
        echo "[migration_verify] pathfind-bridge resolved backend ok: ${path_resolved}"
      fi
      echo "[migration_verify] pathfind-bridge dispatch total ok: ${path_total}"
      echo "[migration_verify] pathfind-bridge checksum observed (non-default iters): ${path_checksum}"
    fi
    if [[ "${path_split}" == "true" ]]; then
      if [[ "${path_iters}" == "100" ]]; then
        run_path_split_and_check \
          "35400.00000" \
          "35400.00000" \
          cargo run -q -p sim-test --release -- --bench-pathfind-bridge-split --iters "${path_iters}" --backend "${path_backend}"
      else
        run_path_split_observe \
          cargo run -q -p sim-test --release -- --bench-pathfind-bridge-split --iters "${path_iters}" --backend "${path_backend}"
      fi
    fi
    if [[ "${path_backend_smoke}" == "true" ]]; then
      run_path_backend_smoke_and_check \
        "${path_backend_smoke_iters}" \
        "${path_backend_smoke_expect_has_gpu}" \
        "${path_backend_smoke_expect_auto}" \
        "${path_backend_smoke_expect_gpu}" \
        cargo run -q -p sim-test --release -- --bench-pathfind-backend-smoke --iters "${path_backend_smoke_iters}"
    fi
    if [[ "${stress_iters}" == "10000" ]]; then
      run_bench_and_check \
        "stress-math" \
        "24032652.00000" \
        cargo run -q -p sim-test --release -- --bench-stress-math --iters "${stress_iters}"
    else
      run_bench_observe \
        "stress-math" \
        cargo run -q -p sim-test --release -- --bench-stress-math --iters "${stress_iters}"
    fi
    STRESS_CHECKSUM="${LAST_BENCH_CHECKSUM}"
    if [[ "${needs_iters}" == "10000" ]]; then
      run_bench_and_check \
        "needs-math" \
        "38457848.00000" \
        cargo run -q -p sim-test --release -- --bench-needs-math --iters "${needs_iters}"
    else
      run_bench_observe \
        "needs-math" \
        cargo run -q -p sim-test --release -- --bench-needs-math --iters "${needs_iters}"
    fi
    NEEDS_CHECKSUM="${LAST_BENCH_CHECKSUM}"

    if [[ -n "${bench_report_json}" ]]; then
      bench_report_out="${bench_report_json}"
      if [[ "${bench_report_out}" != /* ]]; then
        bench_report_out="${ROOT_DIR}/${bench_report_out}"
      fi
      path_resolved_json="null"
      if [[ -n "${path_resolved:-}" ]]; then
        path_resolved_json="\"${path_resolved}\""
      fi
      split_tuple_checksum_json="null"
      split_xy_checksum_json="null"
      split_tuple_total_json="null"
      split_xy_total_json="null"
      if [[ -n "${PATH_SPLIT_TUPLE_CHECKSUM}" ]]; then
        split_tuple_checksum_json="\"${PATH_SPLIT_TUPLE_CHECKSUM}\""
      fi
      if [[ -n "${PATH_SPLIT_XY_CHECKSUM}" ]]; then
        split_xy_checksum_json="\"${PATH_SPLIT_XY_CHECKSUM}\""
      fi
      if [[ -n "${PATH_SPLIT_TUPLE_TOTAL}" ]]; then
        split_tuple_total_json="${PATH_SPLIT_TUPLE_TOTAL}"
      fi
      if [[ -n "${PATH_SPLIT_XY_TOTAL}" ]]; then
        split_xy_total_json="${PATH_SPLIT_XY_TOTAL}"
      fi
      smoke_checksum_json="null"
      smoke_total_json="null"
      smoke_has_gpu_json="null"
      smoke_expect_has_gpu_json="null"
      smoke_resolved_auto_json="null"
      smoke_resolved_gpu_json="null"
      if [[ -n "${PATH_SMOKE_CHECKSUM}" ]]; then
        smoke_checksum_json="\"${PATH_SMOKE_CHECKSUM}\""
      fi
      if [[ -n "${PATH_SMOKE_TOTAL}" ]]; then
        smoke_total_json="${PATH_SMOKE_TOTAL}"
      fi
      if [[ -n "${path_backend_smoke_expect_has_gpu}" ]]; then
        smoke_expect_has_gpu_json="${path_backend_smoke_expect_has_gpu}"
      fi
      if [[ -n "${PATH_SMOKE_HAS_GPU}" ]]; then
        smoke_has_gpu_json="${PATH_SMOKE_HAS_GPU}"
      fi
      if [[ -n "${PATH_SMOKE_RESOLVED_AUTO}" ]]; then
        smoke_resolved_auto_json="\"${PATH_SMOKE_RESOLVED_AUTO}\""
      fi
      if [[ -n "${PATH_SMOKE_RESOLVED_GPU}" ]]; then
        smoke_resolved_gpu_json="\"${PATH_SMOKE_RESOLVED_GPU}\""
      fi
      mkdir -p "$(dirname "${bench_report_out}")"
      bench_generated_at="$(date -u +"%Y-%m-%dT%H:%M:%SZ")"
      cat > "${bench_report_out}" <<EOF
{
  "schema_version": 1,
  "generated_at_utc": "${bench_generated_at}",
  "path_iters": ${path_iters},
  "stress_iters": ${stress_iters},
  "needs_iters": ${needs_iters},
  "path_backend": "${path_backend}",
  "path_split_enabled": ${path_split},
  "path_smoke_enabled": ${path_backend_smoke},
  "path_smoke_expect_has_gpu": ${smoke_expect_has_gpu_json},
  "path": {
    "checksum": "${path_checksum}",
    "total": ${path_total},
    "resolved": ${path_resolved_json}
  },
  "path_split": {
    "tuple_checksum": ${split_tuple_checksum_json},
    "xy_checksum": ${split_xy_checksum_json},
    "tuple_total": ${split_tuple_total_json},
    "xy_total": ${split_xy_total_json}
  },
  "path_smoke": {
    "checksum": ${smoke_checksum_json},
    "total_each": ${smoke_total_json},
    "has_gpu": ${smoke_has_gpu_json},
    "resolved_auto": ${smoke_resolved_auto_json},
    "resolved_gpu": ${smoke_resolved_gpu_json}
  },
  "stress_checksum": "${STRESS_CHECKSUM}",
  "needs_checksum": "${NEEDS_CHECKSUM}"
}
EOF
      echo "[migration_verify] bench report written: ${bench_report_out}"
    fi
  )
  STEP_BENCH_DURATION=$(( $(date +%s) - step_started_epoch ))
fi

if [[ -n "${verify_report_json}" ]]; then
  to_json_string() {
    local raw_value="$1"
    python3 -c 'import json,sys; print(json.dumps(sys.argv[1]))' "${raw_value}"
  }
  to_abs_path() {
    local raw_path="$1"
    if [[ -z "${raw_path}" ]]; then
      echo ""
      return
    fi
    if [[ "${raw_path}" == /* ]]; then
      echo "${raw_path}"
    else
      echo "${ROOT_DIR}/${raw_path}"
    fi
  }
  to_json_opt_path() {
    local raw_path="$1"
    local abs_path
    abs_path="$(to_abs_path "${raw_path}")"
    if [[ -z "${abs_path}" ]]; then
      echo "null"
    else
      to_json_string "${abs_path}"
    fi
  }
  to_json_opt_sha256() {
    local raw_path="$1"
    local abs_path
    abs_path="$(to_abs_path "${raw_path}")"
    if [[ -z "${abs_path}" || ! -f "${abs_path}" ]]; then
      echo "null"
      return
    fi
    local hash=""
    if command -v shasum >/dev/null 2>&1; then
      hash="$(shasum -a 256 "${abs_path}" | awk '{print $1}')"
    elif command -v sha256sum >/dev/null 2>&1; then
      hash="$(sha256sum "${abs_path}" | awk '{print $1}')"
    fi
    if [[ -z "${hash}" ]]; then
      echo "null"
    else
      to_json_string "${hash}"
    fi
  }
  to_json_opt_size_bytes() {
    local raw_path="$1"
    local abs_path
    abs_path="$(to_abs_path "${raw_path}")"
    if [[ -z "${abs_path}" || ! -f "${abs_path}" ]]; then
      echo "null"
      return
    fi
    local size_bytes
    size_bytes="$(wc -c < "${abs_path}" | tr -d ' ')"
    if [[ -z "${size_bytes}" ]]; then
      echo "null"
    else
      echo "${size_bytes}"
    fi
  }
  to_json_opt_mtime_utc() {
    local raw_path="$1"
    local abs_path
    abs_path="$(to_abs_path "${raw_path}")"
    if [[ -z "${abs_path}" || ! -f "${abs_path}" ]]; then
      echo "null"
      return
    fi
    local mtime_utc
    mtime_utc="$(
      python3 -c 'import datetime,os,sys; print(datetime.datetime.utcfromtimestamp(os.path.getmtime(sys.argv[1])).strftime("%Y-%m-%dT%H:%M:%SZ"))' "${abs_path}" 2>/dev/null || true
    )"
    if [[ -z "${mtime_utc}" ]]; then
      echo "null"
    else
      to_json_string "${mtime_utc}"
    fi
  }
  to_json_opt_exists() {
    local raw_path="$1"
    local abs_path
    abs_path="$(to_abs_path "${raw_path}")"
    if [[ -z "${abs_path}" ]]; then
      echo "null"
      return
    fi
    if [[ -f "${abs_path}" ]]; then
      echo "true"
    else
      echo "false"
    fi
  }
  count_artifact_presence() {
    local raw_path="$1"
    local abs_path
    abs_path="$(to_abs_path "${raw_path}")"
    if [[ -z "${abs_path}" ]]; then
      return
    fi
    artifact_expected_count=$((artifact_expected_count + 1))
    if [[ -f "${abs_path}" ]]; then
      artifact_present_count=$((artifact_present_count + 1))
    fi
  }
  to_json_opt_bool_literal() {
    local raw_value="$1"
    if [[ "${raw_value}" == "true" || "${raw_value}" == "false" ]]; then
      echo "${raw_value}"
    else
      echo "null"
    fi
  }
  to_json_opt_int() {
    local raw_value="$1"
    if [[ "${raw_value}" =~ ^[0-9]+$ ]]; then
      echo "${raw_value}"
    else
      echo "null"
    fi
  }
  to_json_opt_int_from_json_file_key() {
    local raw_path="$1"
    local key="$2"
    local abs_path
    abs_path="$(to_abs_path "${raw_path}")"
    if [[ -z "${abs_path}" || ! -f "${abs_path}" ]]; then
      echo "null"
      return
    fi
    local raw_value
    raw_value="$(
      python3 -c 'import json,sys; d=json.load(open(sys.argv[1], encoding="utf-8")); v=d.get(sys.argv[2]); print(v if isinstance(v, int) else "")' "${abs_path}" "${key}" 2>/dev/null || true
    )"
    to_json_opt_int "${raw_value}"
  }
  to_json_opt_int_from_json_file_path() {
    local raw_path="$1"
    local key_path="$2"
    local abs_path
    abs_path="$(to_abs_path "${raw_path}")"
    if [[ -z "${abs_path}" || ! -f "${abs_path}" ]]; then
      echo "null"
      return
    fi
    local raw_value
    raw_value="$(
      python3 -c 'import json,sys; d=json.load(open(sys.argv[1], encoding="utf-8")); v=d
for part in sys.argv[2].split("."):
    if isinstance(v, dict):
        v=v.get(part)
    else:
        v=None
        break
print(v if isinstance(v, int) else "")' "${abs_path}" "${key_path}" 2>/dev/null || true
    )"
    to_json_opt_int "${raw_value}"
  }
  to_json_opt_array_len_from_json_file_key() {
    local raw_path="$1"
    local key="$2"
    local abs_path
    abs_path="$(to_abs_path "${raw_path}")"
    if [[ -z "${abs_path}" || ! -f "${abs_path}" ]]; then
      echo "null"
      return
    fi
    local raw_value
    raw_value="$(
      python3 -c 'import json,sys; d=json.load(open(sys.argv[1], encoding="utf-8")); v=d.get(sys.argv[2]); print(len(v) if isinstance(v, list) else "")' "${abs_path}" "${key}" 2>/dev/null || true
    )"
    to_json_opt_int "${raw_value}"
  }
  to_json_opt_string_from_json_file_key() {
    local raw_path="$1"
    local key="$2"
    local abs_path
    abs_path="$(to_abs_path "${raw_path}")"
    if [[ -z "${abs_path}" || ! -f "${abs_path}" ]]; then
      echo "null"
      return
    fi
    local raw_value
    raw_value="$(
      python3 -c 'import json,sys; d=json.load(open(sys.argv[1], encoding="utf-8")); v=d.get(sys.argv[2]); print(v if isinstance(v, str) else "")' "${abs_path}" "${key}" 2>/dev/null || true
    )"
    to_json_opt_string "${raw_value}"
  }
  to_json_opt_bool_from_json_file_key() {
    local raw_path="$1"
    local key="$2"
    local abs_path
    abs_path="$(to_abs_path "${raw_path}")"
    if [[ -z "${abs_path}" || ! -f "${abs_path}" ]]; then
      echo "null"
      return
    fi
    local raw_value
    raw_value="$(
      python3 -c 'import json,sys; d=json.load(open(sys.argv[1], encoding="utf-8")); v=d.get(sys.argv[2]); print("true" if v is True else ("false" if v is False else ""))' "${abs_path}" "${key}" 2>/dev/null || true
    )"
    to_json_opt_bool_literal "${raw_value}"
  }
  to_json_conflict_key_preview() {
    local raw_path="$1"
    local max_items="$2"
    local abs_path
    abs_path="$(to_abs_path "${raw_path}")"
    if [[ -z "${abs_path}" || ! -f "${abs_path}" ]]; then
      echo "null"
      return
    fi
    local preview_json
    preview_json="$(
      python3 -c 'import json,sys
d=json.load(open(sys.argv[1], encoding="utf-8"))
try:
    limit=max(0,int(sys.argv[2]))
except Exception:
    print("null")
    raise SystemExit(0)
details=d.get("duplicate_details")
if not isinstance(details, dict):
    print("null")
    raise SystemExit(0)
keys=sorted(
    k for k,v in details.items()
    if isinstance(v, dict) and v.get("value_conflict") is True
)
print(json.dumps(keys[:limit]))' "${abs_path}" "${max_items}" 2>/dev/null || true
    )"
    if [[ -z "${preview_json}" ]]; then
      echo "null"
    else
      echo "${preview_json}"
    fi
  }
  to_json_conflict_key_preview_count() {
    local raw_path="$1"
    local max_items="$2"
    local abs_path
    abs_path="$(to_abs_path "${raw_path}")"
    if [[ -z "${abs_path}" || ! -f "${abs_path}" ]]; then
      echo "null"
      return
    fi
    local preview_count
    preview_count="$(
      python3 -c 'import json,sys
d=json.load(open(sys.argv[1], encoding="utf-8"))
try:
    limit=max(0,int(sys.argv[2]))
except Exception:
    print("")
    raise SystemExit(0)
details=d.get("duplicate_details")
if not isinstance(details, dict):
    print("")
    raise SystemExit(0)
keys=sorted(
    k for k,v in details.items()
    if isinstance(v, dict) and v.get("value_conflict") is True
)
print(min(len(keys), limit))' "${abs_path}" "${max_items}" 2>/dev/null || true
    )"
    to_json_opt_int "${preview_count}"
  }
  to_json_zero_is_true() {
    local raw_value="$1"
    if [[ "${raw_value}" =~ ^[0-9]+$ ]]; then
      if [[ "${raw_value}" -eq 0 ]]; then
        echo "true"
      else
        echo "false"
      fi
    else
      echo "null"
    fi
  }
  to_json_three_zeros_is_true() {
    local first="$1"
    local second="$2"
    local third="$3"
    if [[ "${first}" =~ ^[0-9]+$ && "${second}" =~ ^[0-9]+$ && "${third}" =~ ^[0-9]+$ ]]; then
      if [[ "${first}" -eq 0 && "${second}" -eq 0 && "${third}" -eq 0 ]]; then
        echo "true"
      else
        echo "false"
      fi
    else
      echo "null"
    fi
  }
  to_json_leq_ints() {
    local lhs="$1"
    local rhs="$2"
    if [[ "${lhs}" =~ ^[0-9]+$ && "${rhs}" =~ ^[0-9]+$ ]]; then
      if [[ "${lhs}" -le "${rhs}" ]]; then
        echo "true"
      else
        echo "false"
      fi
    else
      echo "null"
    fi
  }
  to_json_bool_and() {
    local has_null="false"
    local value
    for value in "$@"; do
      if [[ "${value}" == "false" ]]; then
        echo "false"
        return
      fi
      if [[ "${value}" != "true" ]]; then
        has_null="true"
      fi
    done
    if [[ "${has_null}" == "true" ]]; then
      echo "null"
    else
      echo "true"
    fi
  }
  to_json_opt_string() {
    local raw_value="$1"
    if [[ -z "${raw_value}" ]]; then
      echo "null"
    else
      to_json_string "${raw_value}"
    fi
  }
  assert_artifact_exists() {
    local artifact_name="$1"
    local raw_path="$2"
    local abs_path
    abs_path="$(to_abs_path "${raw_path}")"
    if [[ -z "${abs_path}" ]]; then
      return
    fi
    if [[ ! -f "${abs_path}" ]]; then
      echo "[migration_verify] required artifact missing: ${artifact_name} -> ${abs_path}" >&2
      exit 1
    fi
  }
  if [[ "${verify_assert_artifacts}" == "true" ]]; then
    assert_artifact_exists "compile_report_json" "${compile_report_json}"
    assert_artifact_exists "audit_report_json" "${audit_report_json}"
    assert_artifact_exists "audit_duplicate_report_json" "${audit_duplicate_report_json}"
    assert_artifact_exists "audit_conflict_markdown" "${audit_conflict_markdown}"
    assert_artifact_exists "audit_key_owner_policy_json" "${audit_key_owner_policy_json}"
    assert_artifact_exists "audit_owner_policy_markdown" "${audit_owner_policy_markdown}"
    assert_artifact_exists "audit_owner_policy_compare_report_json" "${audit_owner_policy_compare_report_json}"
    if [[ "${WITH_BENCHES}" == "true" ]]; then
      assert_artifact_exists "bench_report_json" "${bench_report_json}"
    fi
    echo "[migration_verify] artifact existence checks passed"
  fi
  verify_report_out="$(to_abs_path "${verify_report_json}")"
  mkdir -p "$(dirname "${verify_report_out}")"
  verify_finished_at_utc="$(date -u +"%Y-%m-%dT%H:%M:%SZ")"
  generated_at="${verify_finished_at_utc}"
  verify_finished_epoch="$(date +%s)"
  total_duration_seconds=$((verify_finished_epoch - VERIFY_STARTED_EPOCH))
  git_branch="$(git -C "${ROOT_DIR}" rev-parse --abbrev-ref HEAD 2>/dev/null || true)"
  git_head="$(git -C "${ROOT_DIR}" rev-parse HEAD 2>/dev/null || true)"
  git_dirty="false"
  if [[ -n "$(git -C "${ROOT_DIR}" status --porcelain 2>/dev/null || true)" ]]; then
    git_dirty="true"
  fi
  git_branch_json="$(to_json_opt_string "${git_branch}")"
  git_head_json="$(to_json_opt_string "${git_head}")"
  compile_report_json_value="$(to_json_opt_path "${compile_report_json}")"
  audit_report_json_value="$(to_json_opt_path "${audit_report_json}")"
  audit_duplicate_report_json_value="$(to_json_opt_path "${audit_duplicate_report_json}")"
  audit_conflict_markdown_value="$(to_json_opt_path "${audit_conflict_markdown}")"
  audit_key_owner_policy_json_value="$(to_json_opt_path "${audit_key_owner_policy_json}")"
  audit_owner_policy_markdown_value="$(to_json_opt_path "${audit_owner_policy_markdown}")"
  audit_owner_policy_compare_report_json_value="$(to_json_opt_path "${audit_owner_policy_compare_report_json}")"
  bench_report_json_value="$(to_json_opt_path "${bench_report_json}")"
  compile_report_json_sha256="$(to_json_opt_sha256 "${compile_report_json}")"
  audit_report_json_sha256="$(to_json_opt_sha256 "${audit_report_json}")"
  audit_duplicate_report_json_sha256="$(to_json_opt_sha256 "${audit_duplicate_report_json}")"
  audit_conflict_markdown_sha256="$(to_json_opt_sha256 "${audit_conflict_markdown}")"
  audit_key_owner_policy_json_sha256="$(to_json_opt_sha256 "${audit_key_owner_policy_json}")"
  audit_owner_policy_markdown_sha256="$(to_json_opt_sha256 "${audit_owner_policy_markdown}")"
  audit_owner_policy_compare_report_json_sha256="$(to_json_opt_sha256 "${audit_owner_policy_compare_report_json}")"
  bench_report_json_sha256="$(to_json_opt_sha256 "${bench_report_json}")"
  audit_report_dir_value="$(to_json_opt_path "${audit_report_dir}")"
  audit_compare_key_owner_policy_value="$(to_json_opt_string "${audit_compare_key_owner_policy}")"
  compile_report_json_config_value="$(to_json_opt_path "${compile_report_json}")"
  bench_report_json_config_value="$(to_json_opt_path "${bench_report_json}")"
  bench_path_iters_value="$(to_json_opt_int "${path_iters-}")"
  bench_stress_iters_value="$(to_json_opt_int "${stress_iters-}")"
  bench_needs_iters_value="$(to_json_opt_int "${needs_iters-}")"
  bench_path_backend_value="$(to_json_opt_string "${path_backend-}")"
  bench_path_split_value="$(to_json_opt_bool_literal "${path_split-}")"
  bench_path_backend_smoke_value="$(to_json_opt_bool_literal "${path_backend_smoke-}")"
  bench_path_backend_smoke_iters_value="$(to_json_opt_int "${path_backend_smoke_iters-}")"
  bench_path_backend_smoke_expect_has_gpu_value="$(to_json_opt_bool_literal "${path_backend_smoke_expect_has_gpu-}")"
  bench_path_backend_smoke_expect_auto_value="$(to_json_opt_string "${path_backend_smoke_expect_auto-}")"
  bench_path_backend_smoke_expect_gpu_value="$(to_json_opt_string "${path_backend_smoke_expect_gpu-}")"
  bench_expected_resolved_backend_value="$(to_json_opt_string "${expected_resolved_backend-}")"
  python3_version_value="$(to_json_opt_string "$(python3 --version 2>/dev/null || true)")"
  cargo_version_value="$(to_json_opt_string "$(cargo --version 2>/dev/null || true)")"
  rustc_version_value="$(to_json_opt_string "$(rustc --version 2>/dev/null || true)")"
  host_os_value="$(to_json_opt_string "$(uname -s 2>/dev/null || true)")"
  host_kernel_release_value="$(to_json_opt_string "$(uname -r 2>/dev/null || true)")"
  host_arch_value="$(to_json_opt_string "$(uname -m 2>/dev/null || true)")"
  host_cpu_count_raw=""
  if command -v getconf >/dev/null 2>&1; then
    host_cpu_count_raw="$(getconf _NPROCESSORS_ONLN 2>/dev/null || true)"
  fi
  if [[ -z "${host_cpu_count_raw}" ]] && command -v nproc >/dev/null 2>&1; then
    host_cpu_count_raw="$(nproc 2>/dev/null || true)"
  fi
  if [[ -z "${host_cpu_count_raw}" ]] && command -v sysctl >/dev/null 2>&1; then
    host_cpu_count_raw="$(sysctl -n hw.ncpu 2>/dev/null || true)"
  fi
  host_cpu_count_value="$(to_json_opt_int "${host_cpu_count_raw}")"
  compile_report_json_size="$(to_json_opt_size_bytes "${compile_report_json}")"
  audit_report_json_size="$(to_json_opt_size_bytes "${audit_report_json}")"
  audit_duplicate_report_json_size="$(to_json_opt_size_bytes "${audit_duplicate_report_json}")"
  audit_conflict_markdown_size="$(to_json_opt_size_bytes "${audit_conflict_markdown}")"
  audit_key_owner_policy_json_size="$(to_json_opt_size_bytes "${audit_key_owner_policy_json}")"
  audit_owner_policy_markdown_size="$(to_json_opt_size_bytes "${audit_owner_policy_markdown}")"
  audit_owner_policy_compare_report_json_size="$(to_json_opt_size_bytes "${audit_owner_policy_compare_report_json}")"
  bench_report_json_size="$(to_json_opt_size_bytes "${bench_report_json}")"
  compile_report_json_mtime_utc="$(to_json_opt_mtime_utc "${compile_report_json}")"
  audit_report_json_mtime_utc="$(to_json_opt_mtime_utc "${audit_report_json}")"
  audit_duplicate_report_json_mtime_utc="$(to_json_opt_mtime_utc "${audit_duplicate_report_json}")"
  audit_conflict_markdown_mtime_utc="$(to_json_opt_mtime_utc "${audit_conflict_markdown}")"
  audit_key_owner_policy_json_mtime_utc="$(to_json_opt_mtime_utc "${audit_key_owner_policy_json}")"
  audit_owner_policy_markdown_mtime_utc="$(to_json_opt_mtime_utc "${audit_owner_policy_markdown}")"
  audit_owner_policy_compare_report_json_mtime_utc="$(to_json_opt_mtime_utc "${audit_owner_policy_compare_report_json}")"
  bench_report_json_mtime_utc="$(to_json_opt_mtime_utc "${bench_report_json}")"
  compile_report_json_exists="$(to_json_opt_exists "${compile_report_json}")"
  audit_report_json_exists="$(to_json_opt_exists "${audit_report_json}")"
  audit_duplicate_report_json_exists="$(to_json_opt_exists "${audit_duplicate_report_json}")"
  audit_conflict_markdown_exists="$(to_json_opt_exists "${audit_conflict_markdown}")"
  audit_key_owner_policy_json_exists="$(to_json_opt_exists "${audit_key_owner_policy_json}")"
  audit_owner_policy_markdown_exists="$(to_json_opt_exists "${audit_owner_policy_markdown}")"
  audit_owner_policy_compare_report_json_exists="$(to_json_opt_exists "${audit_owner_policy_compare_report_json}")"
  bench_report_json_exists="$(to_json_opt_exists "${bench_report_json}")"
  audit_parity_issue_count="$(to_json_opt_array_len_from_json_file_key "${audit_report_json}" "parity_issues")"
  audit_duplicate_key_count="$(to_json_opt_int_from_json_file_key "${audit_report_json}" "duplicate_key_count")"
  audit_duplicate_conflict_count="$(to_json_opt_int_from_json_file_key "${audit_report_json}" "duplicate_conflict_count")"
  audit_duplicate_consistent_count="$(to_json_opt_int_from_json_file_key "${audit_report_json}" "duplicate_consistent_count")"
  audit_duplicate_report_key_count="$(to_json_opt_int_from_json_file_key "${audit_report_json}" "duplicate_report_key_count")"
  audit_duplicate_report_conflict_count="$(to_json_opt_int_from_json_file_key "${audit_report_json}" "duplicate_report_conflict_count")"
  audit_duplicate_report_locale="$(to_json_opt_string_from_json_file_key "${audit_report_json}" "duplicate_report_locale")"
  audit_inline_localized_field_count="$(to_json_opt_int_from_json_file_key "${audit_report_json}" "inline_localized_field_count")"
  audit_owner_policy_entry_count="$(to_json_opt_int_from_json_file_key "${audit_report_json}" "owner_policy_entry_count")"
  audit_owner_policy_category_count="$(to_json_opt_int_from_json_file_key "${audit_report_json}" "owner_policy_category_count")"
  audit_owner_policy_missing_duplicate_count="$(to_json_opt_int_from_json_file_key "${audit_report_json}" "owner_policy_missing_duplicate_count")"
  audit_owner_policy_unused_count="$(to_json_opt_int_from_json_file_key "${audit_report_json}" "owner_policy_unused_count")"
  compare_missing_count="$(to_json_opt_int_from_json_file_key "${audit_owner_policy_compare_report_json}" "missing_count")"
  compare_extra_count="$(to_json_opt_int_from_json_file_key "${audit_owner_policy_compare_report_json}" "extra_count")"
  compare_changed_count="$(to_json_opt_int_from_json_file_key "${audit_owner_policy_compare_report_json}" "changed_count")"
  compile_generated_at_utc_from_report="$(to_json_opt_string_from_json_file_key "${compile_report_json}" "generated_at_utc")"
  compile_default_locale_from_report="$(to_json_opt_string_from_json_file_key "${compile_report_json}" "default_locale")"
  compile_supported_locale_count_from_report="$(to_json_opt_array_len_from_json_file_key "${compile_report_json}" "supported_locales")"
  compile_active_key_count_from_report="$(to_json_opt_int_from_json_file_key "${compile_report_json}" "active_key_count")"
  compile_registry_key_count_from_report="$(to_json_opt_int_from_json_file_key "${compile_report_json}" "registry_key_count")"
  compile_max_locale_duplicates_from_report="$(to_json_opt_int_from_json_file_key "${compile_report_json}" "max_locale_duplicates")"
  compile_max_locale_duplicate_conflicts_from_report="$(to_json_opt_int_from_json_file_key "${compile_report_json}" "max_locale_duplicate_conflicts")"
  compile_max_locale_missing_filled_from_report="$(to_json_opt_int_from_json_file_key "${compile_report_json}" "max_locale_missing_filled")"
  compile_max_locale_owner_rule_misses_from_report="$(to_json_opt_int_from_json_file_key "${compile_report_json}" "max_locale_owner_rule_misses")"
  compile_owner_policy_entry_count_from_report="$(to_json_opt_int_from_json_file_key "${compile_report_json}" "owner_policy_entry_count")"
  compile_owner_policy_missing_duplicate_count_from_report="$(to_json_opt_int_from_json_file_key "${compile_report_json}" "owner_policy_missing_duplicate_count")"
  compile_owner_policy_unused_count_from_report="$(to_json_opt_int_from_json_file_key "${compile_report_json}" "owner_policy_unused_count")"
  compile_strict_duplicates_from_report="$(to_json_opt_bool_from_json_file_key "${compile_report_json}" "strict_duplicates")"
  compile_threshold_max_duplicate_key_count="$(to_json_opt_int_from_json_file_path "${compile_report_json}" "thresholds.max_duplicate_key_count")"
  compile_threshold_max_duplicate_conflict_count="$(to_json_opt_int_from_json_file_path "${compile_report_json}" "thresholds.max_duplicate_conflict_count")"
  compile_threshold_max_missing_key_fill_count="$(to_json_opt_int_from_json_file_path "${compile_report_json}" "thresholds.max_missing_key_fill_count")"
  compile_threshold_max_owner_rule_miss_count="$(to_json_opt_int_from_json_file_path "${compile_report_json}" "thresholds.max_owner_rule_miss_count")"
  compile_threshold_max_duplicate_owner_missing_count="$(to_json_opt_int_from_json_file_path "${compile_report_json}" "thresholds.max_duplicate_owner_missing_count")"
  compile_threshold_max_owner_unused_count="$(to_json_opt_int_from_json_file_path "${compile_report_json}" "thresholds.max_owner_unused_count")"
  compile_threshold_duplicate_key_count_ok="$(to_json_leq_ints "${compile_max_locale_duplicates_from_report}" "${compile_threshold_max_duplicate_key_count}")"
  compile_threshold_duplicate_conflict_count_ok="$(to_json_leq_ints "${compile_max_locale_duplicate_conflicts_from_report}" "${compile_threshold_max_duplicate_conflict_count}")"
  compile_threshold_missing_key_fill_count_ok="$(to_json_leq_ints "${compile_max_locale_missing_filled_from_report}" "${compile_threshold_max_missing_key_fill_count}")"
  compile_threshold_owner_rule_miss_count_ok="$(to_json_leq_ints "${compile_max_locale_owner_rule_misses_from_report}" "${compile_threshold_max_owner_rule_miss_count}")"
  compile_threshold_duplicate_owner_missing_count_ok="$(to_json_leq_ints "${compile_owner_policy_missing_duplicate_count_from_report}" "${compile_threshold_max_duplicate_owner_missing_count}")"
  compile_threshold_owner_unused_count_ok="$(to_json_leq_ints "${compile_owner_policy_unused_count_from_report}" "${compile_threshold_max_owner_unused_count}")"
  compile_thresholds_all_ok="$(
    to_json_bool_and \
      "${compile_threshold_duplicate_key_count_ok}" \
      "${compile_threshold_duplicate_conflict_count_ok}" \
      "${compile_threshold_missing_key_fill_count_ok}" \
      "${compile_threshold_owner_rule_miss_count_ok}" \
      "${compile_threshold_duplicate_owner_missing_count_ok}" \
      "${compile_threshold_owner_unused_count_ok}"
  )"
  audit_conflict_preview_limit="${verify_audit_conflict_preview_limit}"
  audit_conflict_key_preview="$(to_json_conflict_key_preview "${audit_report_json}" "${audit_conflict_preview_limit}")"
  audit_conflict_key_preview_count="$(to_json_conflict_key_preview_count "${audit_report_json}" "${audit_conflict_preview_limit}")"
  bench_path_iters_from_report="$(to_json_opt_int_from_json_file_key "${bench_report_json}" "path_iters")"
  bench_stress_iters_from_report="$(to_json_opt_int_from_json_file_key "${bench_report_json}" "stress_iters")"
  bench_needs_iters_from_report="$(to_json_opt_int_from_json_file_key "${bench_report_json}" "needs_iters")"
  bench_path_backend_from_report="$(to_json_opt_string_from_json_file_key "${bench_report_json}" "path_backend")"
  bench_path_split_enabled_from_report="$(to_json_opt_bool_from_json_file_key "${bench_report_json}" "path_split_enabled")"
  bench_path_smoke_enabled_from_report="$(to_json_opt_bool_from_json_file_key "${bench_report_json}" "path_smoke_enabled")"
  bench_path_smoke_expect_has_gpu_from_report="$(to_json_opt_bool_from_json_file_key "${bench_report_json}" "path_smoke_expect_has_gpu")"
  bench_stress_checksum_from_report="$(to_json_opt_string_from_json_file_key "${bench_report_json}" "stress_checksum")"
  bench_needs_checksum_from_report="$(to_json_opt_string_from_json_file_key "${bench_report_json}" "needs_checksum")"
  artifact_expected_count=0
  artifact_present_count=0
  count_artifact_presence "${compile_report_json}"
  count_artifact_presence "${audit_report_json}"
  count_artifact_presence "${audit_duplicate_report_json}"
  count_artifact_presence "${audit_conflict_markdown}"
  count_artifact_presence "${audit_key_owner_policy_json}"
  count_artifact_presence "${audit_owner_policy_markdown}"
  count_artifact_presence "${audit_owner_policy_compare_report_json}"
  count_artifact_presence "${bench_report_json}"
  artifact_missing_count=$((artifact_expected_count - artifact_present_count))
  artifacts_complete_status="false"
  if [[ "${artifact_missing_count}" -eq 0 ]]; then
    artifacts_complete_status="true"
  fi
  audit_parity_clean_status="$(to_json_zero_is_true "${audit_parity_issue_count}")"
  audit_duplicate_conflict_free_status="$(to_json_zero_is_true "${audit_duplicate_conflict_count}")"
  audit_owner_policy_missing_duplicate_clean_status="$(to_json_zero_is_true "${audit_owner_policy_missing_duplicate_count}")"
  audit_owner_policy_unused_clean_status="$(to_json_zero_is_true "${audit_owner_policy_unused_count}")"
  owner_policy_compare_clean_status="$(
    to_json_three_zeros_is_true \
      "${compare_missing_count}" \
      "${compare_extra_count}" \
      "${compare_changed_count}"
  )"
  bench_report_present_when_enabled_status="null"
  if [[ "${WITH_BENCHES}" == "true" ]]; then
    if [[ "${bench_report_json_exists}" == "true" ]]; then
      bench_report_present_when_enabled_status="true"
    elif [[ "${bench_report_json_exists}" == "false" ]]; then
      bench_report_present_when_enabled_status="false"
    fi
  fi
  cat > "${verify_report_out}" <<EOF
{
  "schema_version": 1,
  "generated_at_utc": "${generated_at}",
  "started_at_utc": "${VERIFY_STARTED_AT_UTC}",
  "finished_at_utc": "${verify_finished_at_utc}",
  "root_dir": "${ROOT_DIR}",
  "git_branch": ${git_branch_json},
  "git_head": ${git_head_json},
  "git_dirty": ${git_dirty},
  "with_benches": ${WITH_BENCHES},
  "apply_key_fields": ${APPLY_KEY_FIELDS},
  "strip_inline_fields": ${STRIP_INLINE_FIELDS},
  "assert_artifacts": ${verify_assert_artifacts},
  "total_duration_seconds": ${total_duration_seconds},
  "toolchain": {
    "python3": ${python3_version_value},
    "cargo": ${cargo_version_value},
    "rustc": ${rustc_version_value}
  },
  "host": {
    "os": ${host_os_value},
    "kernel_release": ${host_kernel_release_value},
    "arch": ${host_arch_value},
    "cpu_count": ${host_cpu_count_value}
  },
  "config": {
    "audit_report_dir": ${audit_report_dir_value},
    "audit_compare_key_owner_policy": ${audit_compare_key_owner_policy_value},
    "audit_refresh_key_owner_policy": ${audit_refresh_key_owner_policy},
    "audit_conflict_preview_limit": ${verify_audit_conflict_preview_limit},
    "compile_report_json": ${compile_report_json_config_value},
    "bench_report_json": ${bench_report_json_config_value},
    "bench": {
      "path_iters": ${bench_path_iters_value},
      "stress_iters": ${bench_stress_iters_value},
      "needs_iters": ${bench_needs_iters_value},
      "path_backend": ${bench_path_backend_value},
      "path_split": ${bench_path_split_value},
      "path_backend_smoke": ${bench_path_backend_smoke_value},
      "path_backend_smoke_iters": ${bench_path_backend_smoke_iters_value},
      "path_backend_smoke_expect_has_gpu": ${bench_path_backend_smoke_expect_has_gpu_value},
      "path_backend_smoke_expect_auto": ${bench_path_backend_smoke_expect_auto_value},
      "path_backend_smoke_expect_gpu": ${bench_path_backend_smoke_expect_gpu_value},
      "expected_resolved_backend": ${bench_expected_resolved_backend_value}
    }
  },
  "timings_seconds": {
    "rust_tests": ${STEP_TESTS_DURATION},
    "data_localization_extract": ${STEP_EXTRACT_DURATION},
    "localization_compile": ${STEP_COMPILE_DURATION},
    "localization_audit": ${STEP_AUDIT_DURATION},
    "rust_bench": ${STEP_BENCH_DURATION}
  },
  "artifacts": {
    "compile_report_json": ${compile_report_json_value},
    "audit_report_json": ${audit_report_json_value},
    "audit_duplicate_report_json": ${audit_duplicate_report_json_value},
    "audit_conflict_markdown": ${audit_conflict_markdown_value},
    "audit_key_owner_policy_json": ${audit_key_owner_policy_json_value},
    "audit_owner_policy_markdown": ${audit_owner_policy_markdown_value},
    "audit_owner_policy_compare_report_json": ${audit_owner_policy_compare_report_json_value},
    "bench_report_json": ${bench_report_json_value}
  },
  "artifact_counts": {
    "expected": ${artifact_expected_count},
    "present": ${artifact_present_count},
    "missing": ${artifact_missing_count}
  },
  "audit_summary": {
    "parity_issue_count": ${audit_parity_issue_count},
    "duplicate_key_count": ${audit_duplicate_key_count},
    "duplicate_conflict_count": ${audit_duplicate_conflict_count},
    "duplicate_consistent_count": ${audit_duplicate_consistent_count},
    "duplicate_report_key_count": ${audit_duplicate_report_key_count},
    "duplicate_report_conflict_count": ${audit_duplicate_report_conflict_count},
    "duplicate_report_locale": ${audit_duplicate_report_locale},
    "inline_localized_field_count": ${audit_inline_localized_field_count},
    "owner_policy_entry_count": ${audit_owner_policy_entry_count},
    "owner_policy_category_count": ${audit_owner_policy_category_count},
    "owner_policy_missing_duplicate_count": ${audit_owner_policy_missing_duplicate_count},
    "owner_policy_unused_count": ${audit_owner_policy_unused_count}
  },
  "audit_conflict_preview": {
    "limit": ${audit_conflict_preview_limit},
    "count": ${audit_conflict_key_preview_count},
    "keys": ${audit_conflict_key_preview}
  },
  "owner_policy_compare_summary": {
    "missing_count": ${compare_missing_count},
    "extra_count": ${compare_extra_count},
    "changed_count": ${compare_changed_count}
  },
  "compile_summary": {
    "generated_at_utc": ${compile_generated_at_utc_from_report},
    "default_locale": ${compile_default_locale_from_report},
    "supported_locale_count": ${compile_supported_locale_count_from_report},
    "active_key_count": ${compile_active_key_count_from_report},
    "registry_key_count": ${compile_registry_key_count_from_report},
    "max_locale_duplicates": ${compile_max_locale_duplicates_from_report},
    "max_locale_duplicate_conflicts": ${compile_max_locale_duplicate_conflicts_from_report},
    "max_locale_missing_filled": ${compile_max_locale_missing_filled_from_report},
    "max_locale_owner_rule_misses": ${compile_max_locale_owner_rule_misses_from_report},
    "owner_policy_entry_count": ${compile_owner_policy_entry_count_from_report},
    "owner_policy_missing_duplicate_count": ${compile_owner_policy_missing_duplicate_count_from_report},
    "owner_policy_unused_count": ${compile_owner_policy_unused_count_from_report},
    "strict_duplicates": ${compile_strict_duplicates_from_report}
  },
  "compile_thresholds": {
    "max_duplicate_key_count": ${compile_threshold_max_duplicate_key_count},
    "max_duplicate_conflict_count": ${compile_threshold_max_duplicate_conflict_count},
    "max_missing_key_fill_count": ${compile_threshold_max_missing_key_fill_count},
    "max_owner_rule_miss_count": ${compile_threshold_max_owner_rule_miss_count},
    "max_duplicate_owner_missing_count": ${compile_threshold_max_duplicate_owner_missing_count},
    "max_owner_unused_count": ${compile_threshold_max_owner_unused_count}
  },
  "compile_threshold_status": {
    "duplicate_key_count_ok": ${compile_threshold_duplicate_key_count_ok},
    "duplicate_conflict_count_ok": ${compile_threshold_duplicate_conflict_count_ok},
    "missing_key_fill_count_ok": ${compile_threshold_missing_key_fill_count_ok},
    "owner_rule_miss_count_ok": ${compile_threshold_owner_rule_miss_count_ok},
    "duplicate_owner_missing_count_ok": ${compile_threshold_duplicate_owner_missing_count_ok},
    "owner_unused_count_ok": ${compile_threshold_owner_unused_count_ok},
    "all_ok": ${compile_thresholds_all_ok}
  },
  "bench_summary": {
    "path_iters": ${bench_path_iters_from_report},
    "stress_iters": ${bench_stress_iters_from_report},
    "needs_iters": ${bench_needs_iters_from_report},
    "path_backend": ${bench_path_backend_from_report},
    "path_split_enabled": ${bench_path_split_enabled_from_report},
    "path_smoke_enabled": ${bench_path_smoke_enabled_from_report},
    "path_smoke_expect_has_gpu": ${bench_path_smoke_expect_has_gpu_from_report},
    "stress_checksum": ${bench_stress_checksum_from_report},
    "needs_checksum": ${bench_needs_checksum_from_report}
  },
  "verification_status": {
    "artifacts_complete": ${artifacts_complete_status},
    "audit_parity_clean": ${audit_parity_clean_status},
    "audit_duplicate_conflict_free": ${audit_duplicate_conflict_free_status},
    "audit_owner_policy_missing_duplicate_clean": ${audit_owner_policy_missing_duplicate_clean_status},
    "audit_owner_policy_unused_clean": ${audit_owner_policy_unused_clean_status},
    "owner_policy_compare_clean": ${owner_policy_compare_clean_status},
    "bench_report_present_when_enabled": ${bench_report_present_when_enabled_status},
    "compile_thresholds_all_ok": ${compile_thresholds_all_ok}
  },
  "artifact_sha256": {
    "compile_report_json": ${compile_report_json_sha256},
    "audit_report_json": ${audit_report_json_sha256},
    "audit_duplicate_report_json": ${audit_duplicate_report_json_sha256},
    "audit_conflict_markdown": ${audit_conflict_markdown_sha256},
    "audit_key_owner_policy_json": ${audit_key_owner_policy_json_sha256},
    "audit_owner_policy_markdown": ${audit_owner_policy_markdown_sha256},
    "audit_owner_policy_compare_report_json": ${audit_owner_policy_compare_report_json_sha256},
    "bench_report_json": ${bench_report_json_sha256}
  },
  "artifact_size_bytes": {
    "compile_report_json": ${compile_report_json_size},
    "audit_report_json": ${audit_report_json_size},
    "audit_duplicate_report_json": ${audit_duplicate_report_json_size},
    "audit_conflict_markdown": ${audit_conflict_markdown_size},
    "audit_key_owner_policy_json": ${audit_key_owner_policy_json_size},
    "audit_owner_policy_markdown": ${audit_owner_policy_markdown_size},
    "audit_owner_policy_compare_report_json": ${audit_owner_policy_compare_report_json_size},
    "bench_report_json": ${bench_report_json_size}
  },
  "artifact_mtime_utc": {
    "compile_report_json": ${compile_report_json_mtime_utc},
    "audit_report_json": ${audit_report_json_mtime_utc},
    "audit_duplicate_report_json": ${audit_duplicate_report_json_mtime_utc},
    "audit_conflict_markdown": ${audit_conflict_markdown_mtime_utc},
    "audit_key_owner_policy_json": ${audit_key_owner_policy_json_mtime_utc},
    "audit_owner_policy_markdown": ${audit_owner_policy_markdown_mtime_utc},
    "audit_owner_policy_compare_report_json": ${audit_owner_policy_compare_report_json_mtime_utc},
    "bench_report_json": ${bench_report_json_mtime_utc}
  },
  "artifact_exists": {
    "compile_report_json": ${compile_report_json_exists},
    "audit_report_json": ${audit_report_json_exists},
    "audit_duplicate_report_json": ${audit_duplicate_report_json_exists},
    "audit_conflict_markdown": ${audit_conflict_markdown_exists},
    "audit_key_owner_policy_json": ${audit_key_owner_policy_json_exists},
    "audit_owner_policy_markdown": ${audit_owner_policy_markdown_exists},
    "audit_owner_policy_compare_report_json": ${audit_owner_policy_compare_report_json_exists},
    "bench_report_json": ${bench_report_json_exists}
  }
}
EOF
  echo "[migration_verify] verify report written: ${verify_report_out}"
fi

echo "[migration_verify] completed"
