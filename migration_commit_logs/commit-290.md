# Commit 290 - AgeSystem body 파생치 수식 Rust 브리지 이관

## 커밋 요약
- `age_system`의 body 파생치 계산(speed/strength) 수식을 Rust-first 경로로 이관.

## 상세 변경
- `rust/crates/sim-systems/src/body.rs`
  - 신규 함수 추가:
    - `age_body_speed(agi_realized, speed_scale, speed_base) -> f32`
    - `age_body_strength(str_realized) -> f32`
  - 단위 테스트 추가:
    - speed/strength scaling 결과 검증

- `rust/crates/sim-bridge/src/lib.rs`
  - 신규 GDExtension 메서드 추가:
    - `body_age_body_speed(...)`
    - `body_age_body_strength(...)`

- `scripts/systems/biology/age_system.gd`
  - SimBridge 연결 캐시/조회 로직 추가.
  - 연간 body 재계산 구간의 speed/strength 계산을 Rust-first 호출로 전환(fallback 유지).

## 기능 영향
- 생애주기 갱신 시 반복 계산되는 body 파생치 수식을 Rust 경로로 이전해 계산 비용을 낮출 기반 확보.
- 브리지 실패 시 기존 GDScript 계산 경로를 유지해 동작 안정성 보장.

## 검증
- `cd rust && cargo test -q` 통과.
- `cd rust && cargo run -q -p sim-test` 통과 (`[sim-test] PASS`).

## Rust 전환 잔여량(이번 청크 기준)
- **데이터 로더 축(R-1 core loader set)**: `9/9` 완료, 잔여 `0/9`.
- **시스템 실행 축(브리지 적용, scripts/systems 기준)**: `19/56` 적용, 잔여 `37/56`.
