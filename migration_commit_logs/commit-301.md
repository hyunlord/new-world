# Commit 301 - StatThreshold 판정 수식 Rust 브리지 이관

## 커밋 요약
- `stat_threshold_system`의 threshold 활성/해제 판정 수식을 Rust-first 경로로 이관.

## 상세 변경
- `rust/crates/sim-systems/src/body.rs`
  - 신규 함수 추가:
    - `stat_threshold_is_active(value, threshold, direction_code, hysteresis, currently_active) -> bool`
      - `direction_code`: `0=below`, `1=above`
  - 단위 테스트 추가:
    - `stat_threshold_is_active_handles_below_and_hysteresis`
    - `stat_threshold_is_active_handles_above_and_hysteresis`

- `rust/crates/sim-bridge/src/lib.rs`
  - 신규 GDExtension 메서드 추가:
    - `body_stat_threshold_is_active(...)`

- `scripts/systems/record/stat_threshold_system.gd`
  - SimBridge 캐시/조회 로직 추가.
  - `_check_threshold`를 Rust-first 호출로 전환(fallback 유지).
  - direction 문자열을 Rust 호출용 코드(`below=0`, `above=1`)로 매핑.

## 기능 영향
- threshold 조건 평가의 반복 분기 연산이 Rust 경로로 이동.
- 브리지 실패 시 기존 GDScript 분기 로직으로 fallback 유지.

## 검증
- `cd rust && cargo test -q` 통과.
- `cd rust && cargo run -q -p sim-test` 통과 (`[sim-test] PASS`).

## Rust 전환 잔여량(이번 청크 기준)
- **데이터 로더 축(R-1 core loader set)**: `9/9` 완료, 잔여 `0/9`.
- **시스템 실행 축(브리지 적용, scripts/systems 기준)**: `30/56` 적용, 잔여 `26/56`.
