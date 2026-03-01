# Commit 247 - Smoke GPU capability 기대값 게이트 추가

## 커밋 요약
- `migration_verify` backend smoke 검증에 GPU capability 기대값 환경변수(`MIGRATION_BENCH_PATH_BACKEND_SMOKE_EXPECT_HAS_GPU`)를 추가해, 실행 환경의 feature 상태를 명시적으로 강제 검증 가능하게 확장.

## 상세 변경
- `tools/migration_verify.sh`
  - 신규 환경변수:
    - `MIGRATION_BENCH_PATH_BACKEND_SMOKE_EXPECT_HAS_GPU` (`true|false`, optional)
  - 입력 검증 추가:
    - 값이 설정된 경우 `true|false`만 허용.
  - smoke 파서 확장:
    - mode별 `has_gpu` 값 파싱 후 세 모드 일치 검증.
    - 기대값이 설정된 경우 `has_gpu`와 정확히 일치하는지 강제.
  - 벤치 컨텍스트 로그 확장:
    - `smoke_expect_has_gpu=...` 출력.
  - smoke 호출부 인자 확장:
    - `run_path_backend_smoke_and_check`에 capability 기대값 전달.
  - bench report 확장:
    - `path_smoke_expect_has_gpu` 필드 추가(불리언/null).

## 기능 영향
- GPU feature on/off가 환경별로 섞여도 smoke 검증에서 capability 기대값을 명시해 오탐/누락을 줄일 수 있음.
- CI에서 빌드 변형(cpu-only vs gpu-enabled)별 기대 capability를 선언적으로 강제 가능.

## 검증
- `bash -n tools/migration_verify.sh` 통과.
- `MIGRATION_AUDIT_REPORT_DIR=/tmp/worldsim_audit_artifacts9 MIGRATION_BENCH_PATH_BACKEND_SMOKE=true MIGRATION_BENCH_PATH_BACKEND_SMOKE_ITERS=5 MIGRATION_BENCH_PATH_BACKEND_SMOKE_EXPECT_HAS_GPU=false MIGRATION_BENCH_PATH_BACKEND_SMOKE_EXPECT_AUTO_RESOLVED=cpu MIGRATION_BENCH_PATH_BACKEND_SMOKE_EXPECT_GPU_RESOLVED=cpu tools/migration_verify.sh --with-benches` 통과.
  - smoke 로그에서 `has_gpu=false` 확인.
  - `/tmp/worldsim_audit_artifacts9/bench_report.json`에
    - `path_smoke_expect_has_gpu=false`
    - `path_smoke.has_gpu=false`
    반영 확인.
