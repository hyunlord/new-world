#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
APPLY_KEY_FIELDS="false"
STRIP_INLINE_FIELDS="false"
WITH_BENCHES="false"

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
audit_report_json="${MIGRATION_AUDIT_REPORT_JSON:-}"
audit_duplicate_report_json="${MIGRATION_AUDIT_DUPLICATE_REPORT_JSON:-}"
audit_conflict_markdown="${MIGRATION_AUDIT_CONFLICT_MARKDOWN:-}"
audit_key_owner_policy_json="${MIGRATION_AUDIT_KEY_OWNER_POLICY:-}"
audit_owner_policy_markdown="${MIGRATION_AUDIT_OWNER_POLICY_MARKDOWN:-}"
audit_compare_key_owner_policy="${MIGRATION_AUDIT_COMPARE_KEY_OWNER_POLICY:-}"
audit_refresh_key_owner_policy="${MIGRATION_AUDIT_REFRESH_KEY_OWNER_POLICY:-false}"
audit_report_dir="${MIGRATION_AUDIT_REPORT_DIR:-}"
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
  if [[ "${audit_refresh_key_owner_policy}" == "true" ]]; then
    audit_cmd+=(--refresh-key-owner-policy-auto)
  fi
  audit_cmd+=(--compare-key-owner-policy "${audit_compare_key_owner_policy}")
fi
"${audit_cmd[@]}"

if [[ "${WITH_BENCHES}" == "true" ]]; then
  echo "[migration_verify] 5/5 rust bench checksum verification"
  path_iters="${MIGRATION_BENCH_PATH_ITERS:-100}"
  stress_iters="${MIGRATION_BENCH_STRESS_ITERS:-10000}"
  needs_iters="${MIGRATION_BENCH_NEEDS_ITERS:-10000}"
  path_split="${MIGRATION_BENCH_PATH_SPLIT:-false}"
  path_backend="${MIGRATION_BENCH_PATH_BACKEND:-auto}"
  path_backend_smoke="${MIGRATION_BENCH_PATH_BACKEND_SMOKE:-false}"
  path_backend_smoke_iters="${MIGRATION_BENCH_PATH_BACKEND_SMOKE_ITERS:-10}"
  path_backend_smoke_expect_auto="${MIGRATION_BENCH_PATH_BACKEND_SMOKE_EXPECT_AUTO_RESOLVED:-}"
  path_backend_smoke_expect_gpu="${MIGRATION_BENCH_PATH_BACKEND_SMOKE_EXPECT_GPU_RESOLVED:-}"
  expected_resolved_backend="${MIGRATION_BENCH_EXPECT_RESOLVED_BACKEND:-}"
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
  if [[ -n "${path_backend_smoke_expect_gpu}" && "${path_backend_smoke_expect_gpu}" != "cpu" && "${path_backend_smoke_expect_gpu}" != "gpu" ]]; then
    echo "[migration_verify] MIGRATION_BENCH_PATH_BACKEND_SMOKE_EXPECT_GPU_RESOLVED must be cpu or gpu" >&2
    exit 1
  fi
  echo "[migration_verify] bench iters: path=${path_iters} stress=${stress_iters} needs=${needs_iters} split=${path_split} path_backend=${path_backend} smoke=${path_backend_smoke} smoke_iters=${path_backend_smoke_iters} smoke_expect_auto=${path_backend_smoke_expect_auto:-none} smoke_expect_gpu=${path_backend_smoke_expect_gpu:-none} expected_resolved=${expected_resolved_backend:-none}"

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
    echo "[migration_verify] pathfind-bridge-split dispatch totals ok: tuple=${tuple_total} xy=${xy_total}"
    echo "[migration_verify] pathfind-bridge-split checksums ok: tuple=${tuple_checksum} xy=${xy_checksum}"
  }

  run_path_backend_smoke_and_check() {
    local smoke_iters="$1"
    local expect_auto="$2"
    local expect_gpu="$3"
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
    echo "[migration_verify] pathfind-backend-smoke checksums/dispatch/resolved ok: checksum=${checksum_auto} total_each=${expected_total} resolved_auto=${resolved_auto:-unknown} resolved_gpu=${resolved_gpu:-unknown}"
  }

  (
    cd "${ROOT_DIR}/rust"
    if [[ "${path_iters}" == "100" ]]; then
      path_output="$(cargo run -q -p sim-test --release -- --bench-pathfind-bridge --iters "${path_iters}" --backend "${path_backend}")"
      echo "${path_output}"
      path_checksum="$(echo "${path_output}" | sed -n 's/.*checksum=\([0-9.]*\).*/\1/p' | head -n 1)"
      path_total="$(echo "${path_output}" | sed -n 's/.*total=\([0-9][0-9]*\).*/\1/p' | head -n 1)"
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
  )
fi

echo "[migration_verify] completed"
