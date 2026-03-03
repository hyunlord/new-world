# 0062 - owner transfer allowlist phase 3 (reputation/social-event/morale)

## Commit
- `[rust-r0-162] Expand owner-ready allowlist with reputation social-event and morale`

## 변경 파일
- `scripts/core/simulation/simulation_engine.gd`
  - `_RUST_OWNER_READY_SYSTEM_KEYS`에 3차 allowlist 추가:
    - `reputation_system`
    - `social_event_system`
    - `morale_system`
- `reports/rust-migration/data/runtime-registered-systems-v2.csv`
  - `reputation_system`, `social_event_system`, `morale_system`의 `exec_owner=gdscript -> rust` 반영.
- `reports/rust-migration/data/tracking-metadata.json`
  - `exec_owner_rule`을 phase3 allowlist 기준으로 갱신.
- `reports/rust-migration/README.md`
  - 0062 항목 추가 및 owner transfer 누적률 갱신.

## 변경 API / 시그널 / 스키마
- 공개 GDExtension 시그니처 변경 없음.
- 런타임 오케스트레이션 정책 변경:
  - Rust primary 모드에서 `reputation/social_event/morale`도 GD fallback 스킵 대상 포함.
- tracking 스키마 변경 없음(값만 갱신).

## 검증 결과
- `cd rust && cargo check -p sim-systems -p sim-bridge` ✅
- `cd rust && cargo test -p sim-systems` ✅
- `cd rust && cargo test -p sim-bridge` ✅

## 누적 전환률 (strict 기준)
- State-write 기준 완료율: `8 / 46 = 17.39%`
- Owner transfer 완료율 (`exec_owner=rust`): `8 / 46 = 17.39%`
- State-write 잔여율: `82.61%`
- Owner transfer 잔여율: `82.61%`

## 메모
- 다음 핵심 과제는 `save v2`, `localization fluent/icu`, 잔여 시스템 active-write 포팅.
