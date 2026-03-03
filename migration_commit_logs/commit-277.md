# Commit 277 - Social 핫패스 배치 Rust 이관 (job drift + occupation eval)

## 커밋 요약
- Social 시스템 2곳의 핵심 판단 루프를 Rust 브리지 경로로 확장: `job_satisfaction` drift 배치 스코어링, `occupation` best-skill/hysteresis 판정.

## 상세 변경
- `rust/crates/sim-systems/src/body.rs`
  - 추가 함수:
    - `job_satisfaction_score_batch(...) -> Vec<f32>`
    - `occupation_best_skill_index(...) -> i32`
    - `occupation_should_switch(...) -> bool`
  - 기존 단일 `job_satisfaction_score`를 배치 계산에서 재사용.
  - 단위 테스트 추가:
    - 배치 스코어와 단일 스코어 일치 검증
    - occupation best index/hysteresis 판정 검증

- `rust/crates/sim-bridge/src/lib.rs`
  - 신규 GDExtension 메서드:
    - `body_job_satisfaction_score_batch(...)`
    - `body_occupation_best_skill_index(...)`
    - `body_occupation_should_switch(...)`

- `scripts/systems/social/job_satisfaction_system.gd`
  - job profile runtime cache 추가 (`personality_ideal`, `value_weights`, scalar 필드 packed화).
  - drift 후보 평가를 Rust 배치 메서드(`body_job_satisfaction_score_batch`)로 일괄 계산.
  - Rust 결과 미사용 시 기존 per-profile 계산 fallback 유지.

- `scripts/systems/social/occupation_system.gd`
  - Rust 브리지 연결 추가 (`body_occupation_best_skill_index`, `body_occupation_should_switch`).
  - best skill 선택 및 occupation change hysteresis 판정을 Rust-first로 실행.
  - 브리지 불가 시 기존 GDScript 로직 fallback 유지.

## 기능 영향
- Social 도메인에서 반복 루프(후보 직무 평가, 직업 전환 판정)의 연산 경로가 Rust로 확장.
- fallback 유지로 런타임 회귀 리스크를 낮춘 상태에서 이관 범위를 단계적으로 확대.

## 검증
- `cd rust && cargo test -q` 통과.
- `cd rust && cargo run -q -p sim-test` 통과 (`[sim-test] PASS`).

## Rust 전환 잔여량(이번 청크 기준)
- **데이터 로더 축(R-1 core loader set)**: `9/9` 완료, 잔여 `0/9`.
- **시스템 실행 축(브리지 적용, scripts/systems 기준)**: `6/56` 적용, 잔여 `50/56`.
