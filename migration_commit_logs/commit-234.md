# Commit 234 - Pathfinding backend smoke 매트릭스 검증 추가

## 커밋 요약
- `sim-test`에 backend 모드(`auto/cpu/gpu`) smoke 벤치를 추가하고, `migration_verify`가 모드별 checksum/dispatch total/해석 backend를 자동 검증하도록 확장.

## 상세 변경
- `rust/crates/sim-test/src/main.rs`
  - 신규 CLI 모드 추가:
    - `--bench-pathfind-backend-smoke`
  - `run_pathfind_backend_smoke(args)` 추가:
    - 모드 순회: `auto -> cpu -> gpu`
    - 각 모드에서 backend 설정 + dispatch 카운터 reset 후 pathfind tuple/xy 배치를 반복 실행
    - 모드별 결과 출력:
      - `mode`
      - `configured`
      - `resolved`
      - `iterations`
      - `checksum`
      - `cpu/gpu/total`

- `tools/migration_verify.sh`
  - 신규 환경변수 추가:
    - `MIGRATION_BENCH_PATH_BACKEND_SMOKE` (`true|false`, 기본 `false`)
    - `MIGRATION_BENCH_PATH_BACKEND_SMOKE_ITERS` (양의 정수, 기본 `10`)
  - 입력 검증 확장:
    - smoke 플래그 값 검증
    - smoke iteration 양의 정수 검증
  - bench 컨텍스트 로그 확장:
    - `smoke`, `smoke_iters` 출력
  - `run_path_backend_smoke_and_check(...)` 추가:
    - smoke 출력에서 mode/checksum/total 파싱
    - 3개 모드 라인(`auto/cpu/gpu`) 존재 강제
    - 모드별 checksum 동일성 강제
    - 모드별 dispatch total = `smoke_iters * 2` 강제
    - `mode=cpu`일 때 `resolved=cpu` 강제
    - 실패 시 즉시 종료, 성공 시 요약 로그 출력
  - `--with-benches` 경로에서 smoke 플래그가 켜진 경우 smoke 검증 단계 실행.

## 기능 영향
- pathfinding backend 전환 구조에서 모드별 실행 경로 무결성을 checksum + dispatch 관점으로 빠르게 점검 가능.
- GPU가 아직 CPU fallback이어도 `configured/resolved` 분리 상태를 매트릭스로 지속 검증할 수 있어 이후 실제 GPU 구현 시 회귀 탐지가 쉬워짐.

## 검증
- `cd rust && cargo run -q -p sim-test --release -- --bench-pathfind-backend-smoke --iters 5` 통과.
  - `auto/cpu/gpu` 3모드 출력 확인
  - checksum 동일: `3540.00000`
  - dispatch total 동일: `10` (5 * 2)
- `MIGRATION_BENCH_PATH_BACKEND_SMOKE=true MIGRATION_BENCH_PATH_BACKEND_SMOKE_ITERS=5 tools/migration_verify.sh --with-benches` 통과.
  - smoke 검증 통과 로그 확인
  - 기존 checksum 기준선 유지:
    - pathfind `70800.00000`
    - stress `24032652.00000`
    - needs `38457848.00000`
