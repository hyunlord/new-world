# 0099 - title runtime active-write port

## Commit
- `[rust-r0-199] Port title runtime to active-write and update strict tracking`

## 변경 파일
- `rust/crates/sim-core/src/components/social.rs`
  - `Social` 컴포넌트에 `titles: Vec<String>` 필드 추가 (`serde(default)` 적용).
  - 타이틀 조작 헬퍼 추가:
    - `has_title(&self, title_id: &str) -> bool`
    - `grant_title(&mut self, title_id: &str)`
    - `revoke_title(&mut self, title_id: &str)`
- `rust/crates/sim-systems/src/runtime.rs`
  - `TitleRuntimeSystem` 추가.
  - 실제 write 경로 구현:
    - `Social.titles`에 연령 타이틀(`TITLE_ELDER`) 부여/회수
    - 스킬 레벨 기반 `TITLE_EXPERT_*` / `TITLE_MASTER_*` 부여/회수
    - 정착지 리더 여부 기반 `TITLE_CHIEF` 부여/회수 및 `TITLE_FORMER_CHIEF` 부여
  - 단위 테스트 2건 추가:
    - `title_runtime_system_grants_elder_master_and_chief_titles`
    - `title_runtime_system_revokes_titles_and_marks_former_chief`
- `rust/crates/sim-bridge/src/lib.rs`
  - `title_system` 지원 키 추가.
  - 런타임 등록 경로에 `TitleRuntimeSystem::new(...)` 연결.
  - 지원 시스템 테스트 갱신(`runtime_supports_expected_ported_systems`).
- `reports/rust-migration/data/runtime-registered-systems-v2.csv`
  - `title_system`의 `rust_runtime_impl=no -> yes` 반영 (`exec_owner`는 `gdscript` 유지).
- `reports/rust-migration/data/tracking-metadata.json`
  - `rust_runtime_impl_rule`에 `title_system` 추가.
  - `generated_at` 갱신.
- `reports/rust-migration/README.md`
  - 0099 항목 추가 및 누적 전환률 갱신.

## 추가/삭제 시스템 키
- active-write 구현 추가: `title_system`
- 삭제: 없음

## 변경 API / 시그널 / 스키마
- 공개 GDExtension 시그니처 변경 없음.
- Runtime Rust 구현 범위 확장:
  - `title_system`이 Rust state-write 시스템으로 승격.
- tracking 스키마 변경 없음(값만 갱신).

## 검증 결과
- `cd rust && cargo check -p sim-systems -p sim-bridge` ✅
- `cd rust && cargo test -p sim-systems` ✅ (250 passed)
- `cd rust && cargo test -p sim-bridge` ✅ (28 passed)

## 누적 전환률 (strict 기준)
- State-write 기준 완료율: `28 / 46 = 60.87%`
- Owner transfer 완료율 (`exec_owner=rust`): `27 / 46 = 58.70%`
- State-write 잔여율: `39.13%`
- Owner transfer 잔여율: `41.30%`

## 메모
- 이번 단계로 `title_system`은 no-op이 아닌 실제 `Social.titles` 상태 변경(write) 경로로 Rust 전환됐다.
- 다음 단계는 owner-ready allowlist에 `title_system`을 추가해 실행 소유권을 Rust로 승격하는 것이다.
