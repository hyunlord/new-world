# Commit 040 - stress 수학 벤치 실행 모드 추가

## 커밋 요약
- `sim-test` 실행 바이너리에 stress 수학 경로 마이크로벤치 CLI 모드를 추가.
- 기존 시뮬레이션 실행 흐름은 유지하고, 벤치 옵션 사용 시 빠르게 수학 hot path 성능을 측정 가능하도록 분기.

## 상세 변경
- `rust/crates/sim-test/src/main.rs`
  - 프로그램 시작 시 인자를 파싱해 `--bench-stress-math` 옵션이 있으면 벤치 모드로 진입.
  - 신규 함수:
    - `run_stress_math_bench(args: &[String])`
  - 벤치 입력 옵션:
    - `--iters <N>` (기본값 `200_000`, 최소 1)
  - 벤치 루프에서 현재 Rust stress 수학 함수들을 연속 호출:
    - `stress_continuous_inputs`
    - `stress_appraisal_scale`
    - `stress_emotion_contribution`
    - `stress_recovery_value`
    - `stress_reserve_step`
    - `stress_allostatic_step`
    - `stress_state_snapshot`
    - `stress_resilience_value`
    - `stress_trace_batch_step`
  - `black_box`로 최적화 제거를 방지하고 `Instant`로 총 시간/iteration당 ns를 출력.
  - 출력 포맷:
    - `iterations`, `elapsed_ms`, `ns_per_iter`, `checksum`

## 기능 영향
- stress math Rust 이관 구간을 단일 명령으로 반복 벤치 가능.
- 실 게임 루프 없이 순수 수학 경로 성능 비교(회귀 체크/튜닝)에 활용 가능.
- 기본 실행(옵션 미사용) 동작은 기존과 동일.

## 검증
- `cd rust && cargo fmt -p sim-test -p sim-systems -p sim-bridge` 통과
- `cd rust && cargo test -q -p sim-test` 통과
- `cd rust && cargo test -q -p sim-systems` 통과 (27 tests)
- `cd rust && cargo test -q -p sim-bridge` 통과 (8 tests)
- `cd rust && cargo run -q -p sim-test -- --bench-stress-math --iters 10000` 통과
  - 예시 출력: `ns_per_iter=82.4`
- `tools/migration_verify.sh` 통과
  - strict audit: inline localized fields 0 유지
