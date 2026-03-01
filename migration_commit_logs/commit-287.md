# Commit 287 - SocialEventSystem 핵심 판정 수식 Rust 브리지 이관

## 커밋 요약
- `social_event_system`의 attachment affinity 배율/청혼 수락 확률 계산을 Rust-first 경로로 이관.

## 상세 변경
- `rust/crates/sim-systems/src/body.rs`
  - 신규 함수 추가:
    - `social_attachment_affinity_multiplier(a_mult, b_mult) -> f32`
    - `social_proposal_accept_prob(romantic_interest, compatibility) -> f32`
  - 단위 테스트 추가:
    - affinity multiplier clamp 경계 검증
    - proposal 수락 확률 bounded 검증

- `rust/crates/sim-bridge/src/lib.rs`
  - 신규 GDExtension 메서드 추가:
    - `body_social_attachment_affinity_multiplier(...)`
    - `body_social_proposal_accept_prob(...)`

- `scripts/systems/social/social_event_system.gd`
  - SimBridge 연결 캐시/조회 로직 추가.
  - `_apply_event`의 attachment affinity 배율 계산을 Rust-first 호출로 전환(fallback 유지).
  - `_handle_proposal`의 수락 확률 계산을 Rust-first 호출로 전환(fallback 유지).

## 기능 영향
- 사회 상호작용 핵심 판정 수식 일부가 Rust 경로로 이전되어 반복 계산 비용을 낮출 기반 확보.
- 브리지 비가용/실패 시 기존 GDScript 경로 유지로 런타임 안정성 보장.

## 검증
- `cd rust && cargo test -q` 통과.
- `cd rust && cargo run -q -p sim-test` 통과 (`[sim-test] PASS`).

## Rust 전환 잔여량(이번 청크 기준)
- **데이터 로더 축(R-1 core loader set)**: `9/9` 완료, 잔여 `0/9`.
- **시스템 실행 축(브리지 적용, scripts/systems 기준)**: `16/56` 적용, 잔여 `40/56`.
