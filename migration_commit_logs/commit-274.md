# Commit 274 - sim-data 로더 스키마 sanity 검증 추가

## 커밋 요약
- `sim-data`의 신규 로더(species/mortality/developmental/attachment)에 필드/범위 sanity 검증을 추가하고, 실패 케이스 테스트를 보강.

## 상세 변경
- `rust/crates/sim-data/src/error.rs`
  - `DataError::InvalidField { field, path, reason }` 변형 추가.

- `rust/crates/sim-data/src/species.rs`
  - `validate_species_definition` 추가.
  - `species_id`, `species_name`, 모델 필드(`personality_model`, `emotion_model`, `mortality_model`, `needs_model`), `species_name_key` 공백/누락 검증.

- `rust/crates/sim-data/src/mortality.rs`
  - `validate_mortality_profile`/`required_number` 추가.
  - `model` 비어있음 검증.
  - `baseline`의 `a1/b1/a2/a3/b3` 존재/수치 검증.
  - `tech_modifiers`의 `k1/k2/k3` 존재 및 `>=0` 검증.
  - `care_protection.hunger_min`, `care_protection.protection_factor`의 `[0,1]` 범위 검증.

- `rust/crates/sim-data/src/developmental_stages.rs`
  - `validate_developmental_stages` 추가.
  - stage별 `label_key` 존재/비공백 검증.
  - `age_range`가 `[start,end]` 2요소인지 및 `start < end` 검증.

- `rust/crates/sim-data/src/attachment_config.rs`
  - `validate_attachment_config`/`validate_unit_range` 추가.
  - `determination_window_days > 0` 검증.
  - 민감도/일관성 threshold 4개 필드의 `[0,1]` 범위 검증.
  - `abuser_is_caregiver_ace_min >= 0` 검증.

- `rust/tests/data_loading_test.rs`
  - 임시 디렉터리 기반 invalid JSON 테스트 헬퍼(`TempDirGuard`, `write_json_file`) 추가.
  - 신규 실패 케이스 테스트 4개 추가:
    - empty `species_id`
    - out-of-range mortality `protection_factor`
    - malformed developmental `age_range`
    - out-of-range attachment threshold

## 기능 영향
- 단순 파싱 성공만으로는 통과되지 않고, 기본적인 데이터 무결성 기준을 위반하면 로딩 단계에서 즉시 실패.
- Rust 전환 중 데이터 결함 조기 탐지 능력이 향상.

## 검증
- `cd rust && cargo test -q` 통과.
- `cd rust && cargo run -q -p sim-test` 통과 (`[sim-test] PASS`).

## Rust 전환 잔여량(이번 청크 기준)
- **데이터 로더 축(R-1 core loader set)**: `8/9` 완료, `1/9` 잔여(남은 도메인 로더 1개 + 경로 정리).
- **전체 전환(엔진/시스템 Rust-first 실행 전환)**: 여전히 하이브리드 단계.
