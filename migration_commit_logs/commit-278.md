# Commit 278 - Leader scoring Rust 브리지 이관

## 커밋 요약
- `leader_system`의 핵심 점수 계산(연령 존중도 + composite score)을 Rust-first 경로로 이관.

## 상세 변경
- `rust/crates/sim-systems/src/body.rs`
  - 신규 함수 추가:
    - `leader_age_respect(age_years)`
    - `leader_score(...)`
  - 단위 테스트 추가:
    - 연령 존중도 clamp 검증
    - 평판 보너스 반영 검증

- `rust/crates/sim-bridge/src/lib.rs`
  - 신규 GDExtension 메서드 추가:
    - `body_leader_age_respect(...)`
    - `body_leader_score(...)`

- `scripts/systems/social/leader_system.gd`
  - SimBridge 연결(`_get_sim_bridge`) 추가.
  - `_compute_age_respect`를 Rust-first 호출로 전환(fallback 유지).
  - `_compute_leader_score`의 최종 composite 계산을 Rust-first 호출로 전환(fallback 유지).

## 기능 영향
- Social 리더 선출 점수 계산의 수학 연산 경로가 Rust로 이동.
- 브리지 미사용/실패 시 기존 GDScript 경로를 유지해 안정성 보장.

## 검증
- `cd rust && cargo test -q` 통과.
- `cd rust && cargo run -q -p sim-test` 통과 (`[sim-test] PASS`).

## Rust 전환 잔여량(이번 청크 기준)
- **데이터 로더 축(R-1 core loader set)**: `9/9` 완료, 잔여 `0/9`.
- **시스템 실행 축(브리지 적용, scripts/systems 기준)**: `7/56` 적용, 잔여 `49/56`.
