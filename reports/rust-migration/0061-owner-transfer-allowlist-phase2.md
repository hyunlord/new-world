# 0061 - owner transfer allowlist phase 2 (stress/emotion)

## Commit
- `[rust-r0-161] Expand owner-ready allowlist with stress and emotion systems`

## 변경 파일
- `scripts/core/simulation/simulation_engine.gd`
  - `_RUST_OWNER_READY_SYSTEM_KEYS`에 2차 allowlist 추가:
    - `stress_system`
    - `emotion_system`
- `reports/rust-migration/data/runtime-registered-systems-v2.csv`
  - `stress_system`, `emotion_system`의 `exec_owner=gdscript -> rust` 반영.
- `reports/rust-migration/data/tracking-metadata.json`
  - `exec_owner_rule`을 phase2 allowlist 기준으로 갱신.
- `reports/rust-migration/README.md`
  - 0061 항목 추가 및 owner transfer 누적률 갱신.

## 변경 API / 시그널 / 스키마
- 공개 GDExtension 시그니처 변경 없음.
- 런타임 오케스트레이션 정책 변경:
  - Rust primary 모드에서 `stress_system`, `emotion_system`도 GD fallback 스킵 대상 포함.
- tracking 스키마 변경 없음(값만 갱신).

## 검증 결과
- `cd rust && cargo check -p sim-systems -p sim-bridge` ✅
- `cd rust && cargo test -p sim-systems` ✅
- `cd rust && cargo test -p sim-bridge` ✅

## 누적 전환률 (strict 기준)
- State-write 기준 완료율: `8 / 46 = 17.39%`
- Owner transfer 완료율 (`exec_owner=rust`): `5 / 46 = 10.87%`
- State-write 잔여율: `82.61%`
- Owner transfer 잔여율: `89.13%`

## 메모
- 다음 단계 후보: `reputation/social_event/morale` owner transfer 전 parity 점검 후 allowlist 추가.
