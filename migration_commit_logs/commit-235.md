# Commit 235 - Backend smoke resolved 기대값 검증 확장

## 커밋 요약
- `migration_verify`의 pathfinding backend smoke 검증에 mode별 resolved 기대값(`auto/gpu`) 옵션을 추가해, CPU fallback 환경과 GPU 활성 환경을 모두 명시적으로 게이트할 수 있게 확장.

## 상세 변경
- `tools/migration_verify.sh`
  - 신규 환경변수 추가:
    - `MIGRATION_BENCH_PATH_BACKEND_SMOKE_EXPECT_AUTO_RESOLVED` (`cpu|gpu`, optional)
    - `MIGRATION_BENCH_PATH_BACKEND_SMOKE_EXPECT_GPU_RESOLVED` (`cpu|gpu`, optional)
  - 입력 검증 추가:
    - 두 신규 변수 값이 비어있지 않다면 `cpu|gpu`만 허용.
  - 벤치 컨텍스트 로그 확장:
    - `smoke_expect_auto`, `smoke_expect_gpu` 출력 추가.
  - `run_path_backend_smoke_and_check` 확장:
    - `mode=auto/gpu`의 `resolved` 값을 추가 파싱.
    - 기존 `cpu mode must resolve to cpu` 강제는 유지.
    - 기대값이 설정된 경우 auto/gpu resolved 일치 여부를 추가 검증.
    - 성공 로그에 `resolved_auto`, `resolved_gpu`를 함께 출력.
  - smoke 실행 인자에 expected auto/gpu 값을 전달하도록 호출부 갱신.

## 기능 영향
- 현재처럼 GPU 미활성 빌드에서는 `auto=cpu`, `gpu=cpu` 기대값을 고정해 fallback 회귀를 빠르게 탐지 가능.
- 향후 GPU 빌드에서는 동일 검증 경로에 기대값만 `gpu`로 바꿔 모드별 해석 정책을 즉시 검증 가능.

## 검증
- `bash -n tools/migration_verify.sh` 통과.
- `MIGRATION_BENCH_PATH_BACKEND_SMOKE=true MIGRATION_BENCH_PATH_BACKEND_SMOKE_ITERS=5 MIGRATION_BENCH_PATH_BACKEND_SMOKE_EXPECT_AUTO_RESOLVED=cpu MIGRATION_BENCH_PATH_BACKEND_SMOKE_EXPECT_GPU_RESOLVED=cpu tools/migration_verify.sh --with-benches` 통과.
  - smoke 결과:
    - `resolved_auto=cpu`
    - `resolved_gpu=cpu`
    - checksum `3540.00000`
    - dispatch total `10` (`iters=5` 기준)
  - 기존 checksum 기준선 유지:
    - pathfind `70800.00000`
    - stress `24032652.00000`
    - needs `38457848.00000`
