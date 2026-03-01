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
python3 "${ROOT_DIR}/tools/localization_audit.py" --project-root "${ROOT_DIR}" --strict

if [[ "${WITH_BENCHES}" == "true" ]]; then
  echo "[migration_verify] 5/5 rust bench checksum verification"
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

  (
    cd "${ROOT_DIR}/rust"
    run_bench_and_check \
      "pathfind-bridge" \
      "70800.00000" \
      cargo run -q -p sim-test --release -- --bench-pathfind-bridge --iters 100
    run_bench_and_check \
      "stress-math" \
      "24032652.00000" \
      cargo run -q -p sim-test --release -- --bench-stress-math --iters 10000
    run_bench_and_check \
      "needs-math" \
      "38457848.00000" \
      cargo run -q -p sim-test --release -- --bench-needs-math --iters 10000
  )
fi

echo "[migration_verify] completed"
