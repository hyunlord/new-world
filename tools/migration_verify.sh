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
audit_cmd=(python3 "${ROOT_DIR}/tools/localization_audit.py" --project-root "${ROOT_DIR}" --strict)
if [[ -n "${audit_report_json}" ]]; then
  audit_cmd+=(--report-json "${audit_report_json}")
fi
if [[ -n "${audit_duplicate_report_json}" ]]; then
  audit_cmd+=(--duplicate-report-json "${audit_duplicate_report_json}")
fi
if [[ -n "${audit_conflict_markdown}" ]]; then
  audit_cmd+=(--duplicate-conflict-markdown "${audit_conflict_markdown}")
fi
"${audit_cmd[@]}"

if [[ "${WITH_BENCHES}" == "true" ]]; then
  echo "[migration_verify] 5/5 rust bench checksum verification"
  path_iters="${MIGRATION_BENCH_PATH_ITERS:-100}"
  stress_iters="${MIGRATION_BENCH_STRESS_ITERS:-10000}"
  needs_iters="${MIGRATION_BENCH_NEEDS_ITERS:-10000}"
  path_split="${MIGRATION_BENCH_PATH_SPLIT:-false}"
  for value in "${path_iters}" "${stress_iters}" "${needs_iters}"; do
    if ! [[ "${value}" =~ ^[0-9]+$ ]] || [[ "${value}" -le 0 ]]; then
      echo "[migration_verify] bench iterations must be positive integers" >&2
      exit 1
    fi
  done
  if [[ "${path_split}" != "true" && "${path_split}" != "false" ]]; then
    echo "[migration_verify] MIGRATION_BENCH_PATH_SPLIT must be true or false" >&2
    exit 1
  fi
  echo "[migration_verify] bench iters: path=${path_iters} stress=${stress_iters} needs=${needs_iters} split=${path_split}"

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
    tuple_checksum="$(echo "${checksum_lines}" | sed -n '1p')"
    xy_checksum="$(echo "${checksum_lines}" | sed -n '2p')"
    if [[ -z "${tuple_checksum}" || -z "${xy_checksum}" ]]; then
      echo "[migration_verify] pathfind-bridge-split checksum parse failed" >&2
      exit 1
    fi
    if [[ "${tuple_checksum}" != "${xy_checksum}" ]]; then
      echo "[migration_verify] pathfind-bridge-split checksum mismatch: tuple=${tuple_checksum} xy=${xy_checksum}" >&2
      exit 1
    fi
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
    tuple_checksum="$(echo "${checksum_lines}" | sed -n '1p')"
    xy_checksum="$(echo "${checksum_lines}" | sed -n '2p')"
    if [[ -z "${tuple_checksum}" || -z "${xy_checksum}" ]]; then
      echo "[migration_verify] pathfind-bridge-split checksum parse failed" >&2
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
    echo "[migration_verify] pathfind-bridge-split checksums ok: tuple=${tuple_checksum} xy=${xy_checksum}"
  }

  (
    cd "${ROOT_DIR}/rust"
    if [[ "${path_iters}" == "100" ]]; then
      run_bench_and_check \
        "pathfind-bridge" \
        "70800.00000" \
        cargo run -q -p sim-test --release -- --bench-pathfind-bridge --iters "${path_iters}"
    else
      run_bench_observe \
        "pathfind-bridge" \
        cargo run -q -p sim-test --release -- --bench-pathfind-bridge --iters "${path_iters}"
    fi
    if [[ "${path_split}" == "true" ]]; then
      if [[ "${path_iters}" == "100" ]]; then
        run_path_split_and_check \
          "35400.00000" \
          "35400.00000" \
          cargo run -q -p sim-test --release -- --bench-pathfind-bridge-split --iters "${path_iters}"
      else
        run_path_split_observe \
          cargo run -q -p sim-test --release -- --bench-pathfind-bridge-split --iters "${path_iters}"
      fi
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
