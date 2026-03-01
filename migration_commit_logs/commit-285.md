# Commit 285 - FamilySystem newborn health 수식 Rust 브리지 이관

## 커밋 요약
- `family_system`의 신생아 건강도 계산 수식을 Rust-first 경로로 이관.

## 상세 변경
- `rust/crates/sim-systems/src/body.rs`
  - 신규 함수 추가:
    - `family_newborn_health(gestation_weeks, mother_nutrition, mother_age, genetics_z, tech) -> f32`
  - 단위 테스트 추가:
    - 조산(preterm) 대비 만삭(term) 건강도 우위 검증

- `rust/crates/sim-bridge/src/lib.rs`
  - 신규 GDExtension 메서드 추가:
    - `body_family_newborn_health(...)`

- `scripts/systems/social/family_system.gd`
  - SimBridge 연결 캐시/조회 로직 추가.
  - `_calc_newborn_health`를 Rust-first 호출로 전환(fallback 유지).

## 기능 영향
- 출산 시 건강도 계산 수식이 Rust 경로로 이동해 관련 수치 계산의 일관성과 성능 기반을 강화.
- 브리지 실패 시 기존 GDScript 계산 경로를 유지해 시뮬레이션 안정성 보장.

## 검증
- `cd rust && cargo test -q` 통과.
- `cd rust && cargo run -q -p sim-test` 통과 (`[sim-test] PASS`).

## Rust 전환 잔여량(이번 청크 기준)
- **데이터 로더 축(R-1 core loader set)**: `9/9` 완료, 잔여 `0/9`.
- **시스템 실행 축(브리지 적용, scripts/systems 기준)**: `14/56` 적용, 잔여 `42/56`.
