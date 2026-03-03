# Commit 321 - Psychology coordinator break 타입 코드화 Rust 브리지 이관

## 커밋 요약
- `psychology_coordinator`의 mental break 타입 캐시/복구 경로를 Rust 타입 코드 매핑 기반으로 전환.

## 상세 변경
- `rust/crates/sim-systems/src/body.rs`
  - 신규 함수 추가:
    - `psychology_break_type_code(...) -> i32`
    - `psychology_break_type_label(...) -> &'static str`
  - 단위 테스트 추가:
    - `psychology_break_type_code_roundtrips_known_types`

- `rust/crates/sim-bridge/src/lib.rs`
  - 신규 GDExtension 메서드 추가:
    - `body_psychology_break_type_code(...)`
    - `body_psychology_break_type_label(...)`

- `scripts/systems/psychology/psychology_coordinator.gd`
  - SimBridge 캐시/조회 로직(`_get_sim_bridge`) 추가.
  - `_on_mental_break_started`:
    - break_type 문자열을 Rust 코드로 변환해 캐시하는 Rust-first 경로 추가.
  - `_on_mental_break_recovered`:
    - 캐시된 code를 Rust 라벨로 복원해 coping/morale로 전달하는 Rust-first 경로 추가.
  - 브리지 실패 시 기존 문자열 캐시 fallback 유지.

## 기능 영향
- coordinator의 break type 캐시 표현이 Rust 코드 경유로 정규화됨.
- 기존 signal wiring 및 coping/morale 연계 동작은 유지.

## 검증
- `cd rust && cargo test -q` 통과.
- `cd rust && cargo run -q -p sim-test` 통과 (`[sim-test] PASS`).

## Rust 전환 잔여량(이번 청크 기준)
- **데이터 로더 축(R-1 core loader set)**: `9/9` 완료, 잔여 `0/9`.
- **시스템 실행 축(bridged 대상 56개 기준)**: `54/56` 적용, 잔여 `2/56`.
- **잔여 주요 파일(2)**:
  - `scripts/systems/psychology/coping_system.gd`
  - `scripts/systems/psychology/emotion_system.gd`
