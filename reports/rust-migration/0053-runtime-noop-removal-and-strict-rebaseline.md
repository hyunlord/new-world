# 0053 - Runtime no-op baseline removal + strict state-write rebaseline

## Commit
- `[rust-r0-153] Remove no-op runtime baselines and rebaseline strict tracking`

## 변경 파일
- `rust/crates/sim-systems/src/runtime.rs`
  - `ResourceRegenSystem`, `NeedsRuntimeSystem`, `UpperNeedsRuntimeSystem`만 유지하도록 런타임 시스템 정리.
  - `let _` 기반 no-op baseline 시스템 구현과 `*_baseline_runs_without_side_effects` 테스트 제거.
- `rust/crates/sim-bridge/src/lib.rs`
  - Rust runtime 등록 가능 시스템 목록을 state-write 3개로 축소.
  - 삭제된 시스템의 등록 분기 및 관련 검증 assertions 제거/수정.
- `reports/rust-migration/data/runtime-registered-systems-v2.csv`
  - `rust_runtime_impl`를 state-write 기준으로 재분류 (`yes`: 3개, `no`: 43개).
- `reports/rust-migration/data/tracking-metadata.json`
  - `rust_runtime_impl_rule`를 `runtime_state_write_only_v1(resource_regen_system|needs_system|upper_needs_system)`로 교체.
- `reports/rust-migration/README.md`
  - 0053 항목 추가, legacy 보고치와 strict state-write/owner-transfer 수치 분리 표기.

## 변경 API / 시그널 / 스키마
- GDExtension 외부 시그니처 변경 없음.
- Runtime registration policy 변경:
  - Rust 지원 시스템: `resource_regen_system`, `needs_system`, `upper_needs_system`.
  - 기존 no-op baseline 시스템은 Rust 지원 목록에서 제외.
- Tracking schema 필드 추가/삭제 없음. 값 산정 규칙만 strict로 변경.

## 제거된 no-op baseline 시스템 키
- `stats_recorder`
- `stat_sync_system`
- `stress_system`
- `emotion_system`
- `stat_threshold_system`
- `job_assignment_system`
- `child_stress_processor`
- `mental_break_system`
- `occupation_system`
- `trauma_scar_system`
- `title_system`
- `value_system`
- `network_system`
- `social_event_system`
- `building_effect_system`
- `family_system`
- `leader_system`
- `age_system`
- `mortality_system`
- `population_system`
- `migration_system`
- `trait_violation_system`
- `reputation_system`
- `contagion_system`
- `job_satisfaction_system`
- `morale_system`

## 검증 결과
- `cd rust && cargo check -p sim-systems -p sim-bridge` ✅
- `cd rust && cargo test -p sim-systems` ✅
- `cd rust && cargo test -p sim-bridge` ✅
- `runtime.rs` baseline 패턴 검사 (`*_baseline_runs_without_side_effects`) 0건 ✅

## 누적 전환률 (strict 기준)
- State-write 기준 완료율: `3 / 46 = 6.52%`
- Owner transfer 완료율 (`exec_owner=rust`): `0 / 46 = 0.00%`
- 잔여율: `93.48%`

## 메모
- 본 커밋은 “메트릭 정직화 + no-op 제거” 정리 단계다.
- 다음 단계는 구조 변경(bridge 모듈 분리)과 GPU placeholder 정리 후, 핵심 루프 실포팅(`stress -> emotion -> reputation -> social_event -> morale`)으로 진행한다.
