# Commit 276 - Job Satisfaction 핫패스 Rust 브리지 이관

## 커밋 요약
- `job_satisfaction` 계산 경로를 Rust 브리지(`SimBridge`) 우선으로 전환하고, GDScript fallback을 유지.

## 상세 변경
- `rust/crates/sim-systems/src/body.rs`
  - 신규 함수 `job_satisfaction_score(...) -> f32` 추가.
  - personality/value fit, need fit, 최종 가중합/클램프 로직을 순수 Rust로 구현.
  - 단위 테스트 2개 추가:
    - 가중식 형태 검증
    - 무가중/기본치 케이스 검증

- `rust/crates/sim-bridge/src/lib.rs`
  - 신규 GDExtension 함수 `body_job_satisfaction_score(...) -> f32` 추가.
  - GDScript에서 PackedArray 기반 입력으로 직접 호출 가능.

- `scripts/systems/social/job_satisfaction_system.gd`
  - `_get_sim_bridge()` 추가(메서드 존재 시 캐시).
  - `_compute_satisfaction()`에서 Rust 메서드 호출 경로 추가.
  - Rust 호출 실패/미존재 시 기존 GDScript 계산 로직으로 fallback 유지.

## 기능 영향
- Job satisfaction 계산이 Rust-first 경로를 사용해 반복 연산 비용을 낮출 기반 확보.
- 런타임 안정성을 위해 fallback 경로를 그대로 유지하여 동작 회귀 리스크를 제한.

## 검증
- `cd rust && cargo test -q` 통과.
- `cd rust && cargo run -q -p sim-test` 통과 (`[sim-test] PASS`).

## Rust 전환 잔여량(이번 청크 기준)
- **데이터 로더 축(R-1 core loader set)**: `9/9` 완료, 잔여 `0/9`.
- **시스템 실행 축(브리지 적용 기준)**:
  - `scripts/systems` 내 Rust 브리지 사용 시스템: `5/56`.
  - 잔여: `51/56` 시스템은 여전히 GDScript-only 실행 경로.
