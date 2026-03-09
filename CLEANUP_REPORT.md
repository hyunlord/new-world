# WorldSim 코드 정리 보고서

## 1. A-1 Authoritative Path

- 전환 파일: `rust/crates/sim-bridge/src/lib.rs`, `rust/crates/sim-bridge/src/runtime_registry.rs`, `rust/crates/sim-test/src/main.rs`, `rust/tests/data_loading_test.rs`
- 결과: `DataRegistry::load_from_directory()`가 런타임과 headless 테스트의 authoritative RON 경로가 됨
- RON 경로: `rust/crates/sim-data/data`
- JSON 로더: 병렬 유지. 런타임 호환을 위해 `personality_distribution`와 `name_generator` bootstrap만 legacy JSON에서 계속 로드
- 회귀 방지: 기존 `load_all()` 기반 JSON 테스트는 유지하고, RON registry 전용 테스트를 병행 추가

## 2. GDScript 계획

- `GD_LEGACY_CLEANUP_PLAN.md` 생성
- 즉시 삭제 가능: 1개 파일
- 참조 제거 후 삭제: 58개 파일
- 보류: 30개 파일
- 핵심 결론: `.tscn`보다 `scenes/main/main.gd`, `project.godot` autoload, UI/debug preload가 주된 삭제 블로커

## 3. sim-core 정리

- `Skills.entries`: `HashMap` -> `BTreeMap` 전환
- `Values`: 수동 serde 구현 제거, derive 기반 직렬화/역직렬화로 정리
- `f32` 마킹: 5개 컴포넌트 파일에 `TODO(v3.1)` 추가
- 남아 있는 `pub ...: f32` 필드: 23개
- `Social`: `Vec` 기반 관계 저장소는 영향 범위가 커서 이번 단계에서는 `TODO(v3.1)`만 추가

## 4. config.rs 분류

- 파일 상단에 v2-era 상수 마이그레이션 가이드 추가
- 상위 핵심 상수에 카테고리 주석 추가:
  - Category A (World Rules / A-9)
  - Category B (System Tuning / A-5)
  - Category C (Engine Internal)
- 이번 단계의 분류 TODO 수: 23개

## 5. sim-systems 마킹

- 파일 단위 `REFACTOR` TODO 추가: 9개 runtime 파일
- 시스템 단위 `DELETE` TODO 추가: 7개 시스템
- 전체 `TODO(v3.1)` 마커 수: 19개
- 주요 대상:
  - temperament 파이프라인으로 대체될 biology 계열 시스템
  - room/influence 기반으로 대체될 `BuildingEffectRuntimeSystem`
  - causal log / observation 계층으로 재구성될 chronicle 계열

## 6. 빌드 상태

- `cargo check --workspace`: PASS
- `cargo test --workspace`: PASS
- `cargo clippy --workspace -- -D warnings`: PASS

## 7. 확인된 제한사항

- GDScript legacy는 계획만 수립했고 실제 삭제는 하지 않음
- `config.rs` 501개 상수 전체 분류는 이번 단계 범위를 넘어서므로, 대표 상수만 우선 카테고리화함
- JSON 로더는 완전 제거하지 않음. RON 전환 중 호환 bootstrap 경로로 병행 유지함
