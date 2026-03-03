# 0060 - owner transfer allowlist phase 1 (resource/needs/upper-needs)

## Commit
- `[rust-r0-160] Enable owner-ready allowlist for initial Rust runtime systems`

## 변경 파일
- `scripts/core/simulation/simulation_engine.gd`
  - `_RUST_OWNER_READY_SYSTEM_KEYS`에 1차 allowlist 추가:
    - `resource_regen_system`
    - `needs_system`
    - `upper_needs_system`
  - rust primary + rust_registered 조건에서 GD fallback tick 실행을 스킵하도록 활성화.
- `reports/rust-migration/data/runtime-registered-systems-v2.csv`
  - `resource_regen_system`, `needs_system`, `upper_needs_system`의 `exec_owner=gdscript -> rust` 반영.
- `reports/rust-migration/data/tracking-metadata.json`
  - `exec_owner_rule`을 allowlist 반영 규칙으로 갱신.
- `reports/rust-migration/README.md`
  - 0060 항목 추가 및 owner transfer 누적률 갱신.

## 변경 API / 시그널 / 스키마
- 공개 GDExtension 시그니처 변경 없음.
- 런타임 오케스트레이션 정책 변경:
  - Rust primary 모드에서 allowlist 대상은 GD fallback 실행 경로를 건너뜀.
- tracking 스키마 변경 없음(값만 갱신).

## 검증 결과
- `cd rust && cargo check -p sim-systems -p sim-bridge` ✅
- `cd rust && cargo test -p sim-systems` ✅
- `cd rust && cargo test -p sim-bridge` ✅

## 누적 전환률 (strict 기준)
- State-write 기준 완료율: `8 / 46 = 17.39%`
- Owner transfer 완료율 (`exec_owner=rust`): `3 / 46 = 6.52%`
- State-write 잔여율: `82.61%`
- Owner transfer 잔여율: `93.48%`

## 메모
- 다음 단계에서 `stress/emotion` owner transfer 후보를 parity 확인 후 allowlist에 순차 추가.
