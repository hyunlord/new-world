# Commit 275 - occupations/jobs 도메인 로더 추가

## 커밋 요약
- Rust `sim-data`에 occupations/jobs 도메인 로더를 추가하고 `DataBundle`에 통합.

## 상세 변경
- 신규 파일: `rust/crates/sim-data/src/occupations.rs`
  - `occupation_categories.json` 파싱:
    - `categories`(job category -> occupation list)
    - `default`
  - `jobs/*.json` 파싱:
    - `job_id`, `riasec`, `hexaco_ideal`, `value_weights`, `primary_skill`, `prestige`, `autonomy_level`, `danger_level`, `creativity_level`
  - 검증 로직 추가:
    - `default`/`job_id`/`riasec`/`primary_skill` 필수
    - `job_id`와 파일명 일치 검증
    - `prestige`/`autonomy_level`/`danger_level`/`creativity_level` 범위 `[0,1]`
    - `hexaco_ideal` 값 범위 `[-1,1]`
    - `value_weights` 값 범위 `[0,1]`

- `rust/crates/sim-data/src/lib.rs`
  - `occupations` 모듈 export 추가.
  - `DataBundle`에 `occupation: OccupationData` 추가.
  - `load_all`이 occupation 데이터를 로드하도록 확장.
  - 내부 테스트에 occupation categories/jobs non-empty 검증 추가.

- `rust/crates/sim-test/src/main.rs`
  - 데이터 로드 로그에 `occupation_categories`, `job_profiles` 카운트 추가.

- `rust/tests/data_loading_test.rs`
  - 통합 테스트에 occupation categories/jobs non-empty 및 default 값 검증 추가.
  - invalid 케이스 테스트 추가:
    - job 파일의 `job_id`와 파일명이 다르면 로더 실패.

## 기능 영향
- GDScript에서 직접 소비하던 occupation/job 프로필 데이터를 Rust에서도 일관된 구조로 로딩 가능.
- 직업/직무 관련 시스템 Rust 이식 시 데이터 계층 재사용 기반 확보.

## 검증
- `cd rust && cargo test -q` 통과.
- `cd rust && cargo run -q -p sim-test` 통과 (`[sim-test] PASS`).

## Rust 전환 잔여량(이번 청크 기준)
- **데이터 로더 축(R-1 core loader set)**: `9/9` 완료, 잔여 `0/9`.
- **전체 전환(엔진/30+ 시스템/Godot tick 오케스트레이션 포함)**: 하이브리드 단계 지속.
