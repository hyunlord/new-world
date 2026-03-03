# Commit 246 - Backend smoke에 GPU capability 지표 연동

## 커밋 요약
- `sim-test` backend smoke 출력에 GPU capability(`has_gpu`)를 추가하고, `migration_verify`가 해당 값을 파싱/검증해 resolved 정책과 일관성을 강제하도록 확장.

## 상세 변경
- `rust/crates/sim-bridge/src/lib.rs`
  - 공개 helper 추가:
    - `has_gpu_pathfind_backend() -> bool`
  - 테스트 보강:
    - `public_backend_mode_helpers_roundtrip_and_validate`에서
      `has_gpu_pathfind_backend() == cfg!(feature = "gpu")` 검증 추가.

- `rust/crates/sim-test/src/main.rs`
  - smoke 벤치 출력 확장:
    - `pathfind-backend-smoke` 라인에 `has_gpu=<true|false>` 필드 추가.

- `tools/migration_verify.sh`
  - smoke 파서 확장:
    - mode별 `has_gpu` 파싱/유효성(`true|false`) 검증.
    - 세 모드 간 `has_gpu` 값 일치 강제.
    - `has_gpu` 값과 `resolved(auto/gpu)` 일관성 강제:
      - `has_gpu=true`면 `resolved auto/gpu = gpu`
      - `has_gpu=false`면 `resolved auto/gpu = cpu`
    - BSD `sed` 호환을 위해 `has_gpu` 파싱 정규식을 단순화해 환경 간 안정성 보강.
  - bench report 확장:
    - `path_smoke.has_gpu` 필드 추가(불리언/null).

## 기능 영향
- GPU feature 활성/비활성 빌드에서 backend smoke 출력만으로 capability 상태와 resolve 정책의 정합성을 자동 검증 가능.
- bench report에 capability 정보가 포함되어 환경별 비교/회귀 추적 정확도가 향상.

## 검증
- `bash -n tools/migration_verify.sh` 통과.
- `MIGRATION_AUDIT_REPORT_DIR=/tmp/worldsim_audit_artifacts8 MIGRATION_BENCH_PATH_BACKEND_SMOKE=true MIGRATION_BENCH_PATH_BACKEND_SMOKE_ITERS=5 MIGRATION_BENCH_PATH_BACKEND_SMOKE_EXPECT_AUTO_RESOLVED=cpu MIGRATION_BENCH_PATH_BACKEND_SMOKE_EXPECT_GPU_RESOLVED=cpu tools/migration_verify.sh --with-benches` 통과.
  - smoke 출력:
    - `has_gpu=false`
    - `resolved_auto=cpu`, `resolved_gpu=cpu`
  - `/tmp/worldsim_audit_artifacts8/bench_report.json`에
    - `path_smoke.has_gpu=false` 반영 확인.
