# Commit 319 - Population 출생 게이트 수식 Rust 브리지 이관

## 커밋 요약
- `population_system`의 출생 허용/차단 판정 핵심 게이트(최대 개체수, 최소 인구, 주거 용량, 식량 임계)를 Rust-first 경로로 이관.

## 상세 변경
- `rust/crates/sim-systems/src/body.rs`
  - 신규 함수 추가:
    - `population_housing_cap(...) -> i32`
    - `population_birth_block_code(...) -> i32`
  - 단위 테스트 추가:
    - `population_housing_cap_uses_free_cap_without_shelters`
    - `population_birth_block_code_follows_gate_order`

- `rust/crates/sim-bridge/src/lib.rs`
  - 신규 GDExtension 메서드 추가:
    - `body_population_housing_cap(...)`
    - `body_population_birth_block_code(...)`

- `scripts/systems/biology/population_system.gd`
  - SimBridge 캐시/조회 로직(`_get_sim_bridge`) 추가.
  - 출생 게이트 공통 헬퍼 추가:
    - `_population_birth_block_code(...)`
    - `_population_housing_cap(...)`
  - `_check_births`:
    - 기존 분산된 조건식을 Rust-first block code 호출로 통합.
  - `_log_population_status`:
    - 동일 block code를 사용해 block reason 계산을 일관화.
  - 브리지 실패 시 기존 GDScript 조건식 fallback 유지.

## 기능 영향
- Population 출생 판정의 반복 조건 계산이 Rust 경로로 이동.
- 출생 처리 순서(식량 차감, spawn, settlement 배정)와 기존 동작 의미는 유지.

## 검증
- `cd rust && cargo test -q` 통과.
- `cd rust && cargo run -q -p sim-test` 통과 (`[sim-test] PASS`).

## Rust 전환 잔여량(이번 청크 기준)
- **데이터 로더 축(R-1 core loader set)**: `9/9` 완료, 잔여 `0/9`.
- **시스템 실행 축(bridged 대상 56개 기준)**: `52/56` 적용, 잔여 `4/56`.
- **잔여 주요 파일(4)**:
  - `scripts/systems/psychology/coping_system.gd`
  - `scripts/systems/psychology/emotion_system.gd`
  - `scripts/systems/psychology/psychology_coordinator.gd`
  - `scripts/systems/record/chronicle_system.gd`
